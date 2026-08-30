#!/usr/bin/env python3
"""Fetch OSM/Nominatim island outlines and named places for the KMA clip."""

from __future__ import annotations

import json
import ssl
import time
import urllib.parse
import urllib.request
from pathlib import Path

OUT = Path("/tmp/harvester-data")
OUT.mkdir(parents=True, exist_ok=True)
UA = "Harvester/0.1 (https://github.com/hrgrvs/harvester; OSM attribution)"
CTX = ssl.create_default_context()


def get_json(url: str, dest: Path) -> dict:
    if dest.exists() and dest.stat().st_size > 80:
        return json.loads(dest.read_text())
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    last = None
    for i in range(5):
        try:
            with urllib.request.urlopen(req, context=CTX, timeout=90) as r:
                data = r.read()
            dest.write_bytes(data)
            return json.loads(data)
        except Exception as e:
            last = e
            time.sleep(1.5 * (i + 1))
    raise RuntimeError(f"{url}: {last}")


def nominatim(query: str, dest: Path, polygon: bool = False) -> dict:
    params = {
        "q": query,
        "format": "geojson",
        "limit": "1",
    }
    if polygon:
        params["polygon_geojson"] = "1"
    url = "https://nominatim.openstreetmap.org/search?" + urllib.parse.urlencode(params)
    time.sleep(1.15)
    return get_json(url, dest)


def main() -> None:
    islands = [
        "Kodiak Island, Alaska",
        "Afognak Island, Alaska",
        "Raspberry Island, Alaska",
        "Shuyak Island, Alaska",
        "Sitkalidak Island, Alaska",
        "Sitkinak Island, Alaska",
        "Tugidak Island, Alaska",
        "Whale Island, Kodiak Island Borough, Alaska",
        "Spruce Island, Kodiak Island Borough, Alaska",
        "Uganik Island, Alaska",
        "Amook Island, Alaska",
        "Sitkinak Island, Alaska",
    ]
    island_fc = {"type": "FeatureCollection", "features": []}
    for name in islands:
        slug = name.split(",")[0].lower().replace(" ", "_")
        gj = nominatim(name, OUT / f"osm_{slug}.geojson", polygon=True)
        for f in gj.get("features", []):
            f.setdefault("properties", {})["query"] = name
            island_fc["features"].append(f)
            print(f"island {name} geom={f.get('geometry', {}).get('type')}")

    (OUT / "osm_islands.geojson").write_text(json.dumps(island_fc))

    places = [
        "Kodiak, Alaska",
        "Port Lions, Alaska",
        "Ouzinkie, Alaska",
        "Larsen Bay, Alaska",
        "Karluk, Alaska",
        "Old Harbor, Alaska",
        "Akhiok, Alaska",
        "Chiniak, Alaska",
        "Womens Bay, Alaska",
        "Uganik Bay, Alaska",
        "Uyak Bay, Alaska",
        "Terror Bay, Alaska",
        "Zachar Bay, Alaska",
        "Amook Pass, Alaska",
        "Spiridon Bay, Alaska",
        "Alitak Bay, Alaska",
        "Moser Bay, Kodiak, Alaska",
        "Olga Bay, Kodiak, Alaska",
        "Kitoi Bay, Alaska",
        "Izhut Bay, Alaska",
        "Duck Bay, Alaska",
        "Anton Larsen Bay, Alaska",
        "Kizhuyak Bay, Alaska",
        "Sharatin Bay, Alaska",
        "Halibut Bay, Kodiak Island, Alaska",
        "Ayakulik, Alaska",
        "Cape Alitak, Alaska",
        "Cape Ikolik, Alaska",
        "Cape Karluk, Alaska",
        "Raspberry Cape, Alaska",
        "Cape Chiniak, Alaska",
        "Cape Trinity, Alaska",
        "Low Cape, Alaska",
        "Foul Bay, Afognak, Alaska",
        "Perenosa Bay, Alaska",
        "Kazakof Bay, Alaska",
        "Ugak Bay, Alaska",
        "Chiniak Bay, Alaska",
        "Marmot Bay, Alaska",
        "Kupreanof Strait, Alaska",
        "Shelikof Strait, Alaska",
        "Lazy Bay, Alaska",
        "Deadman Bay, Alaska",
        "Village Islands, Alaska",
        "Uganik Passage, Alaska",
        "Rocky Point, Kodiak Island, Alaska",
        "Cape Igvak, Alaska",
        "Cape Douglas, Alaska",
        "Kilokak Rocks, Alaska",
        "Kiliuda Bay, Alaska",
        "Sitkalidak Strait, Alaska",
        "Pasagshak, Alaska",
        "Saltery Cove, Alaska",
        "Port Bailey, Alaska",
        "Uganik, Alaska",
        "Uyak, Alaska",
    ]
    place_fc = {"type": "FeatureCollection", "features": []}
    for name in places:
        slug = "".join(c if c.isalnum() else "_" for c in name.lower())
        gj = nominatim(name, OUT / f"place_{slug}.geojson", polygon=False)
        n = 0
        for f in gj.get("features", []):
            f.setdefault("properties", {})["query"] = name
            # Prefer OSM display_name / name
            props = f.get("properties") or {}
            disp = props.get("display_name") or name
            osm_name = disp.split(",")[0].strip()
            props["osm_name"] = osm_name
            f["properties"] = props
            place_fc["features"].append(f)
            n += 1
        print(f"place {name} -> {n}")

    (OUT / "osm_places.geojson").write_text(json.dumps(place_fc, indent=2))
    print("islands", len(island_fc["features"]), "places", len(place_fc["features"]))


if __name__ == "__main__":
    main()
