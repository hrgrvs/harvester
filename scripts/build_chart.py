#!/usr/bin/env python3
"""Rasterize OSM coastline into KMA / island / harbor LODs (5 AAC 18.100)."""

from __future__ import annotations

import json
import math
from collections import deque
from pathlib import Path

SRC = Path("/tmp/harvester-data")
DST = Path("/workspace/data")
DST.mkdir(parents=True, exist_ok=True)

# 5 AAC 18.100
KMA = dict(south=55.50, north=58.8517, west=-156.337, east=-150.00)

# Towns that are OSM place=city/town/village. Bays must never be towns.
TOWN_NAMES = {
    "Kodiak",
    "Port Lions",
    "Ouzinkie",
    "Larsen Bay",
    "Karluk",
    "Old Harbor",
    "Akhiok",
    "Chiniak",
}

# Names that are bays/water, never settlements.
FORCE_BAY = {
    "Ugak Bay",
    "Alitak Bay",
    "Uganik Bay",
    "Uyak Bay",
    "Terror Bay",
    "Zachar Bay",
    "Womens Bay",
    "Chiniak Bay",
    "Moser Bay",
    "Olga Bay",
    "Kizhuyak Bay",
    "Anton Larsen Bay",
    "Sharatin Bay",
    "Spiridon Bay",
    "Deadman Bay",
    "Lazy Bay",
    "Kiliuda Bay",
    "Izhut Bay",
    "Duck Bay",
    "Kitoi Bay",
    "Foul Bay",
    "Perenosa Bay",
    "Marmot Bay",
    "Kazakof Bay",
    "Sitkalidak Strait",
    "Uganik Passage",
    "Amook Bay",
}

FORCE_HARBOR = {
    "Saint Paul Harbor",
    "Kodiak Harbor",
    "Shuyak Harbor",
}


def meters_per_deg(lat: float) -> tuple[float, float]:
    mlat = 111_320.0
    mlon = 111_320.0 * math.cos(math.radians(lat))
    return mlon, mlat


def view_size(view: dict, meters: float) -> tuple[int, int]:
    mid = (view["south"] + view["north"]) / 2
    mlon, mlat = meters_per_deg(mid)
    w = int(round((view["east"] - view["west"]) * mlon / meters))
    h = int(round((view["north"] - view["south"]) * mlat / meters))
    return max(40, w), max(30, h)


def lonlat_to_xy(view: dict, w: int, h: int, lon: float, lat: float) -> tuple[float, float]:
    x = (lon - view["west"]) / (view["east"] - view["west"]) * (w - 1)
    y = (view["north"] - lat) / (view["north"] - view["south"]) * (h - 1)
    return x, y


def load_coast() -> list[list[tuple[float, float]]]:
    raw = json.loads((SRC / "osm_coast_ways.json").read_text())
    lines = []
    for el in raw.get("elements", []):
        geom = el.get("geometry") or []
        pts = [(p["lon"], p["lat"]) for p in geom]
        if len(pts) >= 2:
            lines.append(pts)
    return lines


def simplify(pts: list[tuple[float, float]], eps_deg: float) -> list[tuple[float, float]]:
    if len(pts) < 3 or eps_deg <= 0:
        return pts

    def dperp(a, b, p):
        ax, ay = a
        bx, by = b
        px, py = p
        dx, dy = bx - ax, by - ay
        if dx == 0 and dy == 0:
            return math.hypot(px - ax, py - ay)
        t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)))
        return math.hypot(px - (ax + t * dx), py - (ay + t * dy))

    keep = [False] * len(pts)
    keep[0] = keep[-1] = True
    stack = [(0, len(pts) - 1)]
    while stack:
        i, j = stack.pop()
        mx, mi = -1.0, i
        for k in range(i + 1, j):
            d = dperp(pts[i], pts[j], pts[k])
            if d > mx:
                mx, mi = d, k
        if mx > eps_deg:
            keep[mi] = True
            stack.append((i, mi))
            stack.append((mi, j))
    return [p for p, k in zip(pts, keep) if k]


def paint(grid: list[list[str]], x: int, y: int, w: int, h: int, thick: int = 1):
    for dy in range(-thick, thick + 1):
        for dx in range(-thick, thick + 1):
            xx, yy = x + dx, y + dy
            if 0 <= xx < w and 0 <= yy < h:
                grid[yy][xx] = "#"


def draw_line(grid: list[list[str]], x0: float, y0: float, x1: float, y1: float, w: int, h: int, thick: int = 1):
    n = max(int(abs(x1 - x0)), int(abs(y1 - y0)), 1)
    for i in range(n + 1):
        t = i / n
        x = int(round(x0 + (x1 - x0) * t))
        y = int(round(y0 + (y1 - y0) * t))
        paint(grid, x, y, w, h, thick)


def rasterize(view: dict, meters: float, lines: list, eps: float) -> dict:
    w, h = view_size(view, meters)
    # Cap extreme sizes
    if w * h > 220_000:
        scale = math.sqrt(220_000 / (w * h))
        w = max(40, int(w * scale))
        h = max(30, int(h * scale))
    grid = [["~"] * w for _ in range(h)]
    thick = 1 if min(w, h) < 80 else 1
    for pts in lines:
        simp = simplify(pts, eps)
        xy = [lonlat_to_xy(view, w, h, lon, lat) for lon, lat in simp]
        for i in range(len(xy) - 1):
            draw_line(grid, xy[i][0], xy[i][1], xy[i + 1][0], xy[i + 1][1], w, h, thick)

    # Flood ocean from known water only (SE Gulf, east Pacific, mid-Shelikof).
    # Do not seed the west/NW edge — that cuts the Alaska Peninsula mainland.
    seen = [[False] * w for _ in range(h)]
    q = deque()
    water_ll = [
        (-150.2, 55.6),   # SE Gulf
        (-151.2, 56.2),   # south of Sitkalidak
        (-152.0, 56.6),   # east of Kodiak
        (-151.8, 57.9),   # Marmot
        (-154.6, 57.7),   # mid Shelikof
        (-153.0, 56.3),   # south of Tugidak / open
        (-150.4, 58.4),   # NE Gulf
    ]
    seeds = []
    for lon, lat in water_ll:
        x, y = lonlat_to_xy(view, w, h, lon, lat)
        seeds.append((int(round(x)), int(round(y))))
    # south and east frame, not west
    for x in range(w // 2, w, max(1, w // 15)):
        seeds.append((x, h - 1))
    for y in range(0, h, max(1, h // 15)):
        seeds.append((w - 1, y))
    for x, y in seeds:
        if 0 <= x < w and 0 <= y < h and grid[y][x] != "#":
            q.append((x, y))
            seen[y][x] = True
    while q:
        x, y = q.popleft()
        grid[y][x] = "~"
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < w and 0 <= ny < h and not seen[ny][nx] and grid[ny][nx] != "#":
                seen[ny][nx] = True
                q.append((nx, ny))

    # Unreached cells are land (islands / mainland interior)
    for y in range(h):
        for x in range(w):
            if not seen[y][x] and grid[y][x] != "#":
                grid[y][x] = "."

    # Coast hash on land/water boundary
    for y in range(h):
        for x in range(w):
            if grid[y][x] == ".":
                for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < w and 0 <= ny < h and grid[ny][nx] == "~":
                        grid[y][x] = "#"
                        break

    land = sum(row.count(".") + row.count("#") for row in grid)
    print(f"  raster {w}x{h} land={land} ({100*land/(w*h):.1f}%)")
    return {
        "view": view,
        "meters_per_tile": meters,
        "width": w,
        "height": h,
        "tiles": ["".join(r) for r in grid],
    }


def classify(feat: dict) -> tuple[str, int]:
    name = feat["name"]
    kind = (feat.get("kind") or "").lower()
    if name in FORCE_HARBOR or (
        ("harbor" in name.lower() or "harbour" in name.lower())
        and name != "Old Harbor"
        and not name.endswith(" Point")
    ):
        return "harbor", 2
    if kind in {"city", "town", "village", "hamlet"} and name not in FORCE_BAY:
        lod = 0 if kind in {"city", "town", "village"} or name in TOWN_NAMES else 1
        return "town", lod
    if name in FORCE_BAY or name.endswith(" Bay") or name.endswith(" Strait") or name.endswith(" Passage") or name.endswith(" Pass"):
        # Major bays visible at KMA; small ones from island zoom
        major = name in FORCE_BAY or name in {
            "Shelikof Strait",
            "Kupreanof Strait",
        }
        return "bay", 0 if major else 1
    if name in TOWN_NAMES or kind in {"city", "town", "village"}:
        return "town", 0
    if kind in {"hamlet"}:
        return "town", 1
    if "harbour" in kind or "harbor" in name.lower() or "harbour" in name.lower():
        return "harbor", 2
    if kind in {"islet"} or "island" in name.lower() and name not in {"Kodiak Island", "Afognak Island"}:
        return "islet", 2
    if kind in {"cape", "peninsula"} or name.startswith("Cape ") or name.endswith(" Cape") or name.endswith(" Head") or name.endswith(" Point"):
        return "cape", 1
    if kind == "beach":
        return "beach", 2
    if kind == "locality":
        return "locality", 2
    return "named", 1


def load_labels() -> list[dict]:
    raw = json.loads((SRC / "osm_named_features.json").read_text())
    out = []
    for f in raw:
        name = f["name"].strip()
        if not name:
            continue
        # Never promote Ugak / Alitak to towns
        kind, min_lod = classify(f)
        if name in {"Ugak", "Alitak"} and kind == "town":
            continue
        lon, lat = float(f["lon"]), float(f["lat"])
        if not (KMA["west"] <= lon <= KMA["east"] and KMA["south"] <= lat <= KMA["north"]):
            continue
        out.append(
            {
                "name": name,
                "lon": lon,
                "lat": lat,
                "kind": kind,
                "min_lod": min_lod,
                "osm_id": f.get("osm_id"),
            }
        )
    # Amook Pass: include only if OSM returned it
    return out


def pack_coast(lines: list, eps: float, max_pts: int = 80_000) -> list[list[list[float]]]:
    packed = []
    total = 0
    for pts in lines:
        simp = simplify(pts, eps)
        if len(simp) < 2:
            continue
        # 5 decimal degrees ~ 1.1 m
        seq = [[round(lon, 5), round(lat, 5)] for lon, lat in simp]
        packed.append(seq)
        total += len(seq)
        if total >= max_pts:
            break
    print(f"  coast vectors {len(packed)} lines {total} verts")
    return packed


def main() -> None:
    lines = load_coast()
    print("coast ways", len(lines))
    labels = load_labels()
    print("labels", len(labels), "towns", sum(1 for l in labels if l["kind"] == "town"))

    lods = []
    # ~2 km — whole KMA including Mainland / Shelikof
    print("LOD kma 2000m")
    kma = rasterize(KMA, 2000, lines, eps=0.004)
    kma["id"] = "kma"
    lods.append(kma)

    # ~500 m — archipelago + inner Shelikof
    island_view = dict(south=56.48, north=58.70, west=-155.15, east=-151.90)
    print("LOD island 500m")
    island = rasterize(island_view, 500, lines, eps=0.0012)
    island["id"] = "island"
    lods.append(island)

    # Harbor vectors (~30 m simplify) for 80 m viewport raster
    harbor_coast = pack_coast(lines, eps=0.00028)

    payload = {
        "attribution": (
            "© OpenStreetMap contributors (ODbL). Coastline ways natural=coastline "
            "and OSM names only. Clip: Kodiak Management Area 5 AAC 18.100 "
            "(Cape Douglas to Kilokak Rocks / 55°30'N / 150°W). "
            "NOAA NSDE/CUSP harbor shapefiles were not downloadable from this environment; "
            "OSM coastline is the chart source. GEBCO/NOAA DEM not applied "
            "(coverage is NE city/Ouzinkie/Chiniak only)."
        ),
        "kma": KMA,
        "lods": lods,
        "harbor": {
            "id": "harbor",
            "meters_per_tile": 80,
            "coast": harbor_coast,
        },
        "labels": labels,
    }
    dest = DST / "kodiak_map.json"
    dest.write_text(json.dumps(payload, separators=(",", ":")))
    print("wrote", dest, dest.stat().st_size)


if __name__ == "__main__":
    main()
