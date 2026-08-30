#!/usr/bin/env python3
"""Download official OSM / weather sources used to build Harvester data files."""

from __future__ import annotations

import gzip
import json
import ssl
import time
import urllib.request
from pathlib import Path

OUT = Path("/tmp/harvester-data")
OUT.mkdir(parents=True, exist_ok=True)

UA = "Harvester/0.1 (https://github.com/hrgrvs/harvester; research/game data)"
CTX = ssl.create_default_context()


def get(url: str, dest: Path, retries: int = 4) -> Path:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and dest.stat().st_size > 100:
        print(f"cached {dest}")
        return dest
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    last_err = None
    for i in range(retries):
        try:
            with urllib.request.urlopen(req, context=CTX, timeout=90) as r:
                dest.write_bytes(r.read())
            print(f"ok {url} -> {dest} ({dest.stat().st_size} bytes)")
            return dest
        except Exception as e:
            last_err = e
            print(f"retry {i+1} {url}: {e}")
            time.sleep(2**i)
    raise RuntimeError(f"failed {url}: {last_err}")


def nominatim(query: str, dest: Path) -> Path:
    url = (
        "https://nominatim.openstreetmap.org/search?"
        f"q={urllib.parse.quote(query)}&format=geojson&polygon_geojson=1&limit=1"
    )
    time.sleep(1.1)
    return get(url, dest)


def main() -> None:
    import urllib.parse

    # NDBC 46077 Shelikof Strait standard meteorological 2025
    get(
        "https://www.ndbc.noaa.gov/data/historical/stdmet/46077h2025.txt.gz",
        OUT / "46077h2025.txt.gz",
    )

    # PADQ ASOS hourly, Iowa Mesonet (PADQ METAR)
    padq = (
        "https://mesonet.agron.iastate.edu/cgi-bin/request/asos.py?"
        "station=PADQ&data=tmpf&data=dwpf&data=relh&data=drct&data=sknt"
        "&data=gust&data=p01i&data=alti&data=vsby&data=wxcodes&data=feel"
        "&year1=2025&month1=5&day1=1&year2=2025&month2=10&day2=1"
        "&tz=America%2FAnchorage&format=onlycomma&latlon=no&elev=no"
        "&missing=null&trace=T&direct=no&report_type=3"
    )
    get(padq, OUT / "padq_2025.csv")

    islands = [
        "Kodiak Island, Alaska",
        "Afognak Island, Alaska",
        "Raspberry Island, Alaska",
        "Shuyak Island, Alaska",
        "Sitkalidak Island, Alaska",
        "Sitkinak Island, Alaska",
        "Tugidak Island, Alaska",
        "Whale Island, Kodiak, Alaska",
        "Spruce Island, Kodiak, Alaska",
        "Woody Island, Kodiak, Alaska",
        "Long Island, Kodiak, Alaska",
        "Uganik Island, Alaska",
        "Amook Island, Alaska",
    ]
    for name in islands:
        slug = name.split(",")[0].lower().replace(" ", "_")
        nominatim(name, OUT / f"osm_{slug}.geojson")

    # Named populated places and bays (OSM/Nominatim)
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
        "Moser Bay, Alaska",
        "Olga Bay, Alaska",
        "Kitoi Bay, Alaska",
        "Izhut Bay, Alaska",
        "Duck Bay, Alaska",
        "Anton Larsen Bay, Alaska",
        "Kizhuyak Bay, Alaska",
        "Sharatin Bay, Alaska",
        "Halibut Bay, Kodiak, Alaska",
        "Sturgeon River, Kodiak, Alaska",
        "Ayakulik River, Alaska",
        "Cape Alitak, Alaska",
        "Cape Ikolik, Alaska",
        "Cape Karluk, Alaska",
        "Raspberry Cape, Alaska",
        "Cape Chiniak, Alaska",
        "Cape Trinity, Alaska",
        "Low Cape, Alaska",
        "Termination Point, Kodiak, Alaska",
        "Foul Bay, Alaska",
        "Perenosa Bay, Alaska",
        "Kazakof Bay, Alaska",
        "Danger Bay, Alaska",
        "Afognak Bay, Alaska",
        "Pauls Bay, Alaska",
        "Cape Douglas, Alaska",
        "Kilokak Rocks, Alaska",
        "Cape Igvak, Alaska",
        "Sitkalidak Strait, Alaska",
        "Kiliuda Bay, Alaska",
        "Ugak Bay, Alaska",
        "Chiniak Bay, Alaska",
        "Marmot Bay, Alaska",
        "Kupreanof Strait, Alaska",
        "Shelikof Strait, Alaska",
        "Lazy Bay, Alaska",
        "Deadman Bay, Alaska",
        "Portage Bay, Kodiak, Alaska",
        "Village Islands, Alaska",
        "Uganik Passage, Alaska",
        "North Cape, Kodiak, Alaska",
        "Rocky Point, Kodiak, Alaska",
        "Tanner Head, Alaska",
        "Middle Reef, Kodiak, Alaska",
    ]
    features = []
    for name in places:
        slug = name.replace(", ", "_").replace(" ", "_").lower()
        path = nominatim(name, OUT / f"place_{slug}.geojson")
        try:
            gj = json.loads(path.read_text())
            for f in gj.get("features", []):
                props = f.get("properties") or {}
                props["query"] = name
                f["properties"] = props
                features.append(f)
        except Exception as e:
            print(f"skip {name}: {e}")

    (OUT / "osm_places.geojson").write_text(
        json.dumps({"type": "FeatureCollection", "features": features}, indent=2)
    )
    print(f"places {len(features)}")


if __name__ == "__main__":
    main()
