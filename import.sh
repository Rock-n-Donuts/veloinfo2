#!/usr/bin/bash
osm2pgsql -H db -U postgres -d carte -O flex -S import.lua quebec-latest.osm.pbf