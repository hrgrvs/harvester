#!/usr/bin/env python3
"""Fetch OSM coastline and named marine features for the KMA clip (5 AAC 18.100)."""

from __future__ import annotations

import json
import ssl
import time
import urllib.parse
import urllib.request
from pathlib import Path

OUT = Path("/tmp/harvester-data")
OUT.mkdir(parents=True, exist_ok=True)
UA = "Harvester/0.1 (https://github.com/hrgrvs/harvester; OSM ODbL attribution)"
CTX = ssl.create_default_context()
OVERPASS = "https://overpass-api.de/api/interpreter"

# 5 AAC 18.100 Kodiak Management Area
KMA = dict(s=55.50, n=58.8517, w=-156.337, e=-150.00)

# Tiles so Overpass does not time out on the full KMA.
TILES = [
    (55.50, 57.15, -156.337, -153.20),  # SW: Alitak / Tugidak / Sitkinak / peninsula
    (55.50, 57.15, -153.20, -150.00),  # SE: Sitkalidak / east
    (57.15, 58.10, -156.337, -153.20),  # CW: Uyak / Uganik / Shelikof
    (57.15, 58.10, -153.20, -150.00),  # CE: Kodiak city / Chiniak / Ugak
    (58.10, 58.8517, -156.337, -153.20),  # NW: Afognak west / Cape Douglas mainland
    (58.10, 58.8517, -153.20, -150.00),  # NE: Shuyak / Perenosa / Marmot
]


def post_overpass(query: str, dest: Path, retries: int = 5) -> dict:
    if dest.exists() and dest.stat().st_size > 200:
        print(f"cached {dest} ({dest.stat().st_size})")
        return json.loads(dest.read_text())
    data = urllib.parse.urlencode({"data": query}).encode()
    req = urllib.request.Request(
        OVERPASS,
        data=data,
        headers={"User-Agent": UA, "Content-Type": "application/x-www-form-urlencoded"},
        method="POST",
    )
    last = None
    for i in range(retries):
        try:
            with urllib.request.urlopen(req, context=CTX, timeout=180) as r:
                raw = r.read()
            dest.write_bytes(raw)
            print(f"ok {dest.name} {len(raw)} bytes")
            return json.loads(raw)
        except Exception as e:
            last = e
            print(f"retry {i+1} {dest.name}: {e}")
            time.sleep(8 * (i + 1))
    raise RuntimeError(f"overpass failed {dest}: {last}")


def coastline_query(s, n, w, e) -> str:
    return f"""
[out:json][timeout:120];
(
  way["natural"="coastline"]({s},{w},{n},{e});
);
out geom;
"""


def names_query(s, n, w, e) -> str:
    return f"""
[out:json][timeout:120];
(
  node["place"~"^(city|town|village|hamlet)$"]({s},{w},{n},{e});
  node["natural"~"^(bay|cape|beach|islet|strait|peninsula)$"]({s},{w},{n},{e});
  node["harbour"]({s},{w},{n},{e});
  node["seamark:type"="harbour"]({s},{w},{n},{e});
  node["leisure"="harbour"]({s},{w},{n},{e});
  way["harbour"]({s},{w},{n},{e});
  way["leisure"="harbour"]({s},{w},{n},{e});
  way["natural"~"^(bay|cape|beach|islet)$"]({s},{w},{n},{e});
  way["place"="islet"]({s},{w},{n},{e});
  node["place"="islet"]({s},{w},{n},{e});
  node["place"="locality"]["name"]({s},{w},{n},{e});
  node["natural"="water"]["name"]({s},{w},{n},{e});
);
out center tags;
"""


def main() -> None:
    coasts = []
    for i, (s, n, w, e) in enumerate(TILES):
        gj = post_overpass(coastline_query(s, n, w, e), OUT / f"coast_tile_{i}.json")
        coasts.append(gj)
        time.sleep(2)

    ways = {}
    for gj in coasts:
        for el in gj.get("elements", []):
            if el.get("type") == "way" and el.get("geometry"):
                ways[el["id"]] = el
    (OUT / "osm_coast_ways.json").write_text(json.dumps({"elements": list(ways.values())}))
    print("coast ways", len(ways))

    names = []
    seen = set()
    for i, (s, n, w, e) in enumerate(TILES):
        gj = post_overpass(names_query(s, n, w, e), OUT / f"names_tile_{i}.json")
        for el in gj.get("elements", []):
            tags = el.get("tags") or {}
            name = tags.get("name")
            if not name or name in seen:
                continue
            lat = el.get("lat")
            lon = el.get("lon")
            if lat is None:
                c = el.get("center") or {}
                lat, lon = c.get("lat"), c.get("lon")
            if lat is None or lon is None:
                continue
            seen.add(name)
            names.append(
                {
                    "name": name,
                    "lat": lat,
                    "lon": lon,
                    "osm_type": el.get("type"),
                    "osm_id": el.get("id"),
                    "kind": tags.get("natural")
                    or tags.get("place")
                    or tags.get("harbour")
                    or tags.get("leisure")
                    or tags.get("seamark:type")
                    or "named",
                    "tags": {k: tags[k] for k in tags if k in ("name", "place", "natural", "harbour", "leisure")},
                }
            )
        time.sleep(2)
    (OUT / "osm_named_features.json").write_text(json.dumps(names, indent=2))
    print("named", len(names))
    for n in names:
        if any(x in n["name"].lower() for x in ("paul", "women", "uganik", "uyak", "amook", "alitak", "ugak", "chiniak", "ouzinkie")):
            print(" ", n["kind"], n["name"], n["lat"], n["lon"])


if __name__ == "__main__":
    main()
