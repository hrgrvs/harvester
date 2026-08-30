# Harvester

You have one Kodiak salmon permit and a fish camp. The year is 2025. The water is the Kodiak Management Area and nowhere else.

This is a local, single-player, Dwarf Fortress-style terminal game: a real OpenStreetMap chart of the archipelago, ADF&G openings from the 2025 emergency orders, and run strength from the 2025 season summary. You pick a site. You keep a crew fed. You sit out closures on the beach, not in town. Otters foul the net. Sea lions take fish off the boat. A whale pod can kill the engine.

It is a commercial fishing-empire TUI, not a village-life sim. You are not required to live in Kodiak, Port Lions, Larsen Bay, Old Harbor, Akhiok, or Karluk. The camp is the job.

**Gear.** Set gillnet (S04K) is legal only in the Central Section of the Northwest Kodiak District — the Uganik, Uyak, Amook Pass, Terror, and Zachar beaches — and in inner Alitak until 4 September (5 AAC 18.330). Purse seine (S01K) is mobile. Beach seine is not the default; fewer than three of those permits fished the KMA in 2025.

**Openings.** 2025 ADF&G EOs. The westside was closed 1 June through 5 July for Chinook conservation. Alitak had a 33-hour period 9–10 June, then a long Frazer-driven hole. Pinks were the second-largest on record; 105-hour weekly periods ran in July and time was extended into late August.

**Fish counts.** The almanac shows official KMA season totals from the 29 October 2025 season summary. There is no 2025 Annual Management Report in this tree. The game does not invent week-by-district harvest. What you catch is one permit, clearly labeled as a simulation.

**Weather.** PADQ (Kodiak Airport), NDBC 46077 (Shelikof Strait), and NWS coastal-waters zones PKZ132 / PKZ138 (CWFAER). Chart names are OSM names only.

**Chart.** Real OSM coastline for the whole Kodiak Management Area (5 AAC 18.100), including Mainland and Shelikof — not a cartoon island. Zoom with `+` / `-`. World view is ~2 km per cell; island view ~500 m; harbor view rebuilds at ~80 m so St. Paul Harbor, Womens Bay, and the westside beaches keep their shape. Ugak and Alitak are bays, not towns.

Native terminal app for Omarchy/Arch (home), macOS, and Windows. After install you just run `harvester`.

## Play

```
n          new season
hjkl       walk the chart (HJKL jumps)
+  =       zoom in  (KMA → island → harbor; harbor is a fresh ~80 m raster)
-  _       zoom out (harbor → island → whole KMA, including Mainland/Shelikof)
f          fish (if that section is open and your gear is legal)
c          camp / mend — keep the crew here during closures
.          wait a day
d          deliver the hold
t          town (only on an OSM village or city)
m          move fish camp to the nearest legal site
e          crew / supplies — names, sleep, hunger, motivation, camp stores
a          official 2025 almanac
?          help
q          quit
```

Too much town or playtime and the crew quit. No food and they quit. A closure is for mending gear on the beach. After a long opener they need sleep — exhausted crew fish worse and walk. `e` opens the crew panel (visible bars, not a log dump). Hands are game characters on a small Kodiak fish-camp crew (skipper plus relatives), not real 2025 permit holders.

## Install

One pasteable command per platform. These clone **this** public GitHub repository. No password. No Origin URL.

### macOS / Linux (Omarchy, Arch, …)

```bash
git clone https://github.com/hrgrvs/harvester.git && cd harvester && ./install.sh
```

Then:

```bash
harvester
```

`install.sh` builds a release binary and puts `harvester` on `~/.local/bin` (override with `HARVESTER_PREFIX`). If `cargo` is missing it installs Rust via rustup.

Need Rust yourself first?

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows (PowerShell)

```powershell
git clone https://github.com/hrgrvs/harvester.git; cd harvester; .\install.ps1
```

Then run `harvester`. `install.ps1` builds a release binary and copies `harvester.exe` to `%USERPROFILE%\.local\bin`.

### From a clone you already have

```bash
cargo run --release
```

## Development

Development continues on Origin. This public GitHub repository is the password-free clone for players.

Official sources used in the data files are listed in [`data/SOURCES.md`](data/SOURCES.md). Chart: OSM `natural=coastline` and OSM names only (© OpenStreetMap contributors, ODbL).

## License

MIT. ADF&G and NOAA data remain their own. OSM data is © OpenStreetMap contributors, ODbL.
