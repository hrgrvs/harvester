#!/usr/bin/env python3
"""Build compact game data from official PADQ, NDBC 46077, and OSM extracts."""

from __future__ import annotations

import csv
import datetime as dt
import json
import math
from collections import defaultdict
from pathlib import Path

import netCDF4
import numpy as np

SRC = Path("/tmp/harvester-data")
DST = Path("/workspace/data")
DST.mkdir(parents=True, exist_ok=True)

# KMA clip 5 AAC 18.100
KMA = dict(south=55.50, north=58.8517, west=-156.337, east=-150.00)
# Playable archipelago view (still inside KMA)
VIEW = dict(south=56.45, north=58.65, west=-155.05, east=-151.95)

W, H = 220, 150
SEASON_START = dt.date(2025, 6, 1)
SEASON_END = dt.date(2025, 9, 15)

FILL = 1e35


def valid(x: float) -> bool:
    return x is not None and np.isfinite(x) and abs(x) < 1e30


def ms_to_kt(ms: float) -> float:
    return ms * 1.94384


def c_to_f(c: float) -> float:
    return c * 9 / 5 + 32


def compass(deg: float) -> str:
    dirs = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
            "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"]
    return dirs[int((deg + 11.25) / 22.5) % 16]


def pkz_headline(wind_kt: float, seas_ft: float) -> str:
    """NWS-style marine headline from observed 46077 wind/seas for PKZ138."""
    if wind_kt >= 34 or seas_ft >= 13:
        return "PKZ138 Shelikof Strait: GALE CONDITIONS (from 46077 obs)"
    if wind_kt >= 22 or seas_ft >= 8:
        return "PKZ138 Shelikof Strait: SMALL CRAFT ADVISORY CONDITIONS (from 46077 obs)"
    if wind_kt >= 15 or seas_ft >= 5:
        return "PKZ132 Kodiak / PKZ138: CHOPPY, SCA NEAR (from 46077 obs)"
    return "PKZ132 Kodiak Island waters: LIGHT TO MODERATE (from 46077 obs)"


def build_weather() -> None:
    daily: dict[str, dict] = {}

    # PADQ
    padq_by_day: dict[str, list] = defaultdict(list)
    with (SRC / "padq_2025.csv").open() as f:
        for row in csv.DictReader(f):
            valid_s = row.get("valid") or ""
            if not valid_s:
                continue
            day = valid_s[:10]
            rec = {}
            for k in ("tmpf", "sknt", "gust", "drct", "vsby", "alti"):
                try:
                    rec[k] = float(row[k]) if row.get(k) not in (None, "", "null") else None
                except ValueError:
                    rec[k] = None
            rec["wxcodes"] = row.get("wxcodes") or ""
            padq_by_day[day].append(rec)

    # NDBC
    ds = netCDF4.Dataset(SRC / "46077h2025.nc")
    times = ds.variables["time"][:]
    wind = ds.variables["wind_spd"][:].reshape(-1)
    gust = ds.variables["gust"][:].reshape(-1)
    wdir = ds.variables["wind_dir"][:].reshape(-1)
    atmp = ds.variables["air_temperature"][:].reshape(-1)
    sst = ds.variables["sea_surface_temperature"][:].reshape(-1)
    whgt = ds.variables["wave_height"][:].reshape(-1)
    ndbc_by_day: dict[str, list] = defaultdict(list)
    for i, t in enumerate(times):
        utc = dt.datetime.fromtimestamp(float(t), dt.timezone.utc)
        # Alaska
        local = utc.astimezone(dt.timezone(dt.timedelta(hours=-8)))
        day = local.date().isoformat()
        ndbc_by_day[day].append(
            dict(wind=wind[i], gust=gust[i], wdir=wdir[i], atmp=atmp[i], sst=sst[i], wh=whgt[i])
        )

    d = SEASON_START
    while d <= SEASON_END:
        key = d.isoformat()
        prow = padq_by_day.get(key, [])
        brow = ndbc_by_day.get(key, [])

        def mean(rows, field, pred=valid):
            xs = [float(r[field]) for r in rows if pred(r.get(field))]
            return sum(xs) / len(xs) if xs else None

        def mx(rows, field, pred=valid):
            xs = [float(r[field]) for r in rows if pred(r.get(field))]
            return max(xs) if xs else None

        tmpf = mean(prow, "tmpf")
        sknt = mean(prow, "sknt")
        gust_p = mx(prow, "gust")
        drct = mean(prow, "drct")
        vsby = mean(prow, "vsby")
        wx = next((r["wxcodes"] for r in prow if r.get("wxcodes") and r["wxcodes"] != "null"), "")

        wind_ms = mean(brow, "wind")
        gust_ms = mx(brow, "gust")
        wdir_b = mean(brow, "wdir")
        atmp_c = mean(brow, "atmp")
        sst_c = mean(brow, "sst")
        wh_m = mean(brow, "wh")

        wind_kt = ms_to_kt(wind_ms) if wind_ms is not None else (sknt or 10.0)
        seas_ft = (wh_m * 3.28084) if wh_m is not None else max(2.0, wind_kt * 0.22)
        tmp = tmpf if tmpf is not None else (c_to_f(atmp_c) if atmp_c is not None else 50.0)
        wdir_use = wdir_b if wdir_b is not None else (drct if drct is not None else 225.0)

        fishable = not (wind_kt >= 30 or seas_ft >= 10)
        daily[key] = {
            "date": key,
            "padq_temp_f": round(tmp, 1),
            "padq_wind_kt": round(sknt, 1) if sknt is not None else None,
            "padq_gust_kt": round(gust_p, 1) if gust_p is not None else None,
            "padq_dir": compass(drct) if drct is not None else None,
            "padq_vsby_mi": round(vsby, 1) if vsby is not None else None,
            "padq_wx": wx or None,
            "ndbc_wind_kt": round(ms_to_kt(wind_ms), 1) if wind_ms is not None else None,
            "ndbc_gust_kt": round(ms_to_kt(gust_ms), 1) if gust_ms is not None else None,
            "ndbc_dir": compass(wdir_b) if wdir_b is not None else None,
            "ndbc_seas_ft": round(seas_ft, 1) if wh_m is not None else None,
            "ndbc_air_f": round(c_to_f(atmp_c), 1) if atmp_c is not None else None,
            "ndbc_sst_f": round(c_to_f(sst_c), 1) if sst_c is not None else None,
            "cwfaer": pkz_headline(wind_kt, seas_ft),
            "wind_kt": round(wind_kt, 1),
            "seas_ft": round(seas_ft, 1),
            "dir": compass(wdir_use),
            "temp_f": round(tmp, 1),
            "fishable": fishable,
            "sources": "PADQ ASOS (Iowa Mesonet); NDBC 46077 Shelikof Strait; CWFAER PKZ headlines derived from 46077 obs",
        }
        d += dt.timedelta(days=1)

    (DST / "weather_2025.json").write_text(json.dumps(daily, indent=2))
    print("weather days", len(daily))


def lonlat_to_xy(lon: float, lat: float, view=VIEW) -> tuple[int, int]:
    x = int((lon - view["west"]) / (view["east"] - view["west"]) * (W - 1))
    y = int((view["north"] - lat) / (view["north"] - view["south"]) * (H - 1))
    return x, y


def point_in_ring(x: float, y: float, ring: list) -> bool:
    # ray cast
    inside = False
    n = len(ring)
    j = n - 1
    for i in range(n):
        xi, yi = ring[i][0], ring[i][1]
        xj, yj = ring[j][0], ring[j][1]
        if ((yi > y) != (yj > y)) and (x < (xj - xi) * (y - yi) / (yj - yi + 1e-18) + xi):
            inside = not inside
        j = i
    return inside


def fill_polygon(grid: list[list[str]], coords, char: str) -> None:
    # coords is list of rings; first is outer
    if not coords:
        return
    outer = coords[0]
    holes = coords[1:]
    xs = [p[0] for p in outer]
    ys = [p[1] for p in outer]
    minx, maxx = max(0, int(min(xs))), min(W - 1, int(max(xs)))
    miny, maxy = max(0, int(min(ys))), min(H - 1, int(max(ys)))
    for y in range(miny, maxy + 1):
        for x in range(minx, maxx + 1):
            if point_in_ring(x + 0.5, y + 0.5, outer) and not any(
                point_in_ring(x + 0.5, y + 0.5, h) for h in holes
            ):
                grid[y][x] = char


def project_coords(coords, view=VIEW):
    out = []
    for ring in coords:
        pr = []
        for lon, lat, *rest in ring:
            x, y = lonlat_to_xy(lon, lat, view)
            pr.append((x, y))
        out.append(pr)
    return out


def add_geom(grid, geom, char="."):
    t = geom.get("type")
    coords = geom.get("coordinates")
    if t == "Polygon":
        fill_polygon(grid, project_coords(coords), char)
    elif t == "MultiPolygon":
        for poly in coords:
            fill_polygon(grid, project_coords(poly), char)
    elif t == "LineString":
        pts = [lonlat_to_xy(lon, lat) for lon, lat, *r in coords]
        for i in range(len(pts) - 1):
            line(grid, pts[i], pts[i + 1], char)
    elif t == "MultiLineString":
        for linec in coords:
            pts = [lonlat_to_xy(lon, lat) for lon, lat, *r in linec]
            for i in range(len(pts) - 1):
                line(grid, pts[i], pts[i + 1], char)


def line(grid, a, b, char):
    x0, y0 = a
    x1, y1 = b
    n = max(abs(x1 - x0), abs(y1 - y0), 1)
    for i in range(n + 1):
        x = int(x0 + (x1 - x0) * i / n)
        y = int(y0 + (y1 - y0) * i / n)
        if 0 <= x < W and 0 <= y < H:
            grid[y][x] = char


def build_map() -> None:
    grid = [["~"] * W for _ in range(H)]
    islands_path = SRC / "osm_islands.geojson"
    places_path = SRC / "osm_places.geojson"
    if not islands_path.exists():
        print("WARN no islands yet")
        islands = {"features": []}
    else:
        islands = json.loads(islands_path.read_text())
    if not places_path.exists():
        places = {"features": []}
    else:
        places = json.loads(places_path.read_text())

    for f in islands.get("features", []):
        geom = f.get("geometry") or {}
        add_geom(grid, geom, ".")

    # Coastline hash on land/water boundary
    for y in range(H):
        for x in range(W):
            if grid[y][x] == ".":
                for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < W and 0 <= ny < H and grid[ny][nx] == "~":
                        grid[y][x] = "#"
                        break

    labels = []
    seen = set()
    for f in places.get("features", []):
        geom = f.get("geometry") or {}
        props = f.get("properties") or {}
        name = props.get("osm_name") or (props.get("display_name") or "").split(",")[0]
        name = name.strip()
        if not name or name in seen:
            continue
        # Only keep OSM names (Nominatim display name first token)
        if geom.get("type") != "Point":
            # centroid of polygon if needed
            continue
        lon, lat = geom["coordinates"][:2]
        if not (VIEW["west"] <= lon <= VIEW["east"] and VIEW["south"] <= lat <= VIEW["north"]):
            # still keep if in KMA for mainland labels
            if not (KMA["west"] <= lon <= KMA["east"] and KMA["south"] <= lat <= KMA["north"]):
                continue
        x, y = lonlat_to_xy(lon, lat)
        seen.add(name)
        labels.append(
            {
                "name": name,
                "lon": lon,
                "lat": lat,
                "x": x,
                "y": y,
                "osm_id": props.get("osm_id") or props.get("place_id"),
            }
        )

    rows = ["".join(r) for r in grid]
    payload = {
        "attribution": "© OpenStreetMap contributors. Names and island outlines from Nominatim/OSM. Clip: KMA 5 AAC 18.100.",
        "kma": KMA,
        "view": VIEW,
        "width": W,
        "height": H,
        "legend": { "~": "water", ".": "land", "#": "coast" },
        "tiles": rows,
        "labels": labels,
    }
    (DST / "kodiak_map.json").write_text(json.dumps(payload))
    print("map", W, H, "labels", len(labels), "land", sum(row.count(".") + row.count("#") for row in rows))


if __name__ == "__main__":
    build_weather()
    build_map()
