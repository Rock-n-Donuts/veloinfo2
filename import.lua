local cycleway = osm2pgsql.define_way_table("cycleway_way", {{
    column = 'name',
    type = 'text'
}, {
    column = 'geom',
    type = 'LineString',
    not_null = true
}, {
    column = 'source',
    type = 'int8',
    not_null = true
}, {
    column = 'target',
    type = 'int8',
    not_null = true
}, {
    column = 'kind',
    type = 'text',
    not_null = true
}, {
    column = 'tags',
    type = 'jsonb',
    not_null = true
}, {
    column = 'nodes',
    sql_type = 'int8[] NOT NULL'
}})

local all_way = osm2pgsql.define_way_table("all_way", {{
    column = 'name',
    type = 'text'
}, {
    column = 'geom',
    type = 'LineString',
    not_null = true
}, {
    column = 'source',
    type = 'int8',
    not_null = true
}, {
    column = 'target',
    type = 'int8',
    not_null = true
}, {
    column = 'tags',
    type = 'jsonb',
    not_null = true
}, {
    column = 'nodes',
    sql_type = 'int8[] NOT NULL'
}, {
    column = 'landuse',
    type = 'text'
}, {
    column = 'tunnel',
    type = 'text'
}, {
    column = 'bridge',
    type = 'text'
}})

local landuse = osm2pgsql.define_table({
    name = 'landuse',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'multipolygon',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'landuse',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local landcover = osm2pgsql.define_table({
    name = 'landcover',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'multipolygon',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'landuse',
        type = 'text'
    }, {
        column = 'natural',
        type = 'text'
    }, {
        column = 'leisure',
        type = 'text'
    }, {
        column = 'landcover',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local landcover_far = osm2pgsql.define_table({
    name = 'landcover_far',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'multipolygon',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'landuse',
        type = 'text'
    }, {
        column = 'natural',
        type = 'text'
    }, {
        column = 'leisure',
        type = 'text'
    }, {
        column = 'landcover',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local water_name = osm2pgsql.define_table({
    name = 'water_name',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'point',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'place',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local aeroway = osm2pgsql.define_table({
    name = 'aeroway',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'aeroway',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local transportation = osm2pgsql.define_table({
    name = 'transportation',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'name_fr',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'tunnel',
        type = 'text'
    }, {
        column = 'highway',
        type = 'text'
    }, {
        column = 'railway',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local building = osm2pgsql.define_table({
    name = 'building',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'building',
        type = 'text'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local boundary = osm2pgsql.define_table({
    name = 'boundary',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = {{
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
        not_null = true
    }, {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    }, {
        column = 'boundary',
        type = 'text'
    }, {
        column = 'admin_level',
        type = 'integer'
    }},
    indexes = {{
        column = 'geom',
        method = 'gist'
    }}
})

local all_node = osm2pgsql.define_node_table('all_node', {{
    column = 'name',
    type = 'text'
}, {
    column = 'geom',
    type = 'Point'
}, {
    column = 'tags',
    type = 'jsonb'
}, {
    column = 'place',
    type = 'text'
}})

function osm2pgsql.process_way(object)
    if object.tags.highway == 'cycleway' or object.tags.cycleway == "track" or object.tags["cycleway:left"] == "track" or
        object.tags["cycleway:right"] == "track" or object.tags["cycleway:both"] == "track" then
        cycleway:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            kind = 'cycleway',
            tags = object.tags,
            nodes = "{" .. table.concat(object.nodes, ",") .. "}"
        })
    elseif object.tags.bicycle == "designated" or object.tags["cycleway:left"] == "share_busway" or
        object.tags["cycleway:right"] == "share_busway" or object.tags["cycleway:both"] == "share_busway" or
        object.tags["cycleway:right"] == "lane" or object.tags["cycleway:left"] == "lane" or
        object.tags["cycleway:both"] == "lane" then
        cycleway:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            kind = 'designated',
            tags = object.tags,
            nodes = " {" .. table.concat(object.nodes, ",") .. "}"
        })
    elseif object.tags.cycleway == "shared_lane" or object.tags.cycleway == "lane" or object.tags["cycleway:left"] ==
        "shared_lane" or object.tags["cycleway:right"] == "shared_lane" or object.tags["cycleway:both"] == "shared_lane" or
        object.tags["bicycle"] == "yes" then
        cycleway:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            kind = 'shared_lane',
            tags = object.tags,
            nodes = "{" .. table.concat(object.nodes, ",") .. "}"
        })
    end

    if object.tags.building then
        building:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            tags = object.tags,
            building = object.tags.building
        })
    end

    if object.tags.highway or object.tags.railway then
        transportation:insert({
            name = object.tags.name,
            name_fr = object.tags["name:fr"],
            geom = object:as_linestring(),
            tags = object.tags,
            tunnel = object.tags.tunnel,
            highway = object.tags.highway,
            railway = object.tags.railway

        })
    end

    if object.tags.aeroway then
        aeroway:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            tags = object.tags,
            aeroway = object.tags.aeroway
        })
    end

    if object.tags["bicycle"] == "yes" or object.tags.tunnel or object.tags.highway or object.tags.bridge then
        all_way:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            tags = object.tags,
            nodes = "{" .. table.concat(object.nodes, ",") .. "}",
            tunnel = object.tags.tunnel,
            highway = object.tags.highway,
            bridge = object.tags.bridge
        })
    end

    if object.is_closed and
        (object.tags.landuse == "forest" or object.tags.landuse == "cemetery" or object.tags.natural == "wood" or
            object.tags.natural == "water" or object.tags.leisure == "park" or object.tags.landuse == "residential") then
        landcover:insert({
            name = object.tags.name,
            geom = object:as_polygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            landcover = object.tags.landcover
        })
    end

    if object.is_closed and object.as_polygon():area() > 1e-3 and
        (object.tags.landuse == "forest" or object.tags.landuse == "cemetery" or object.tags.natural == "wood" or
            object.tags.natural == "water" or object.tags.leisure == "park" or object.tags.landuse == "residential") then
        landcover_far:insert({
            name = object.tags.name,
            geom = object:as_polygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            landcover = object.tags.landcover
        })
    end
end

function osm2pgsql.process_relation(object)
    if object.tags.landuse == "forest" or object.tags.landuse == "cemetery" or object.tags.natural == "wood" or
        object.tags.natural == "water" or object.tags.leisure == "park" or object.tags.landuse == "residential" then
        landcover:insert({
            name = object.tags.name,
            geom = object:as_multipolygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            landcover = object.tags.landcover
        })
    end
    if object:as_multipolygon():area() > 1e-3 and
        (object.tags.landuse == "forest" or object.tags.landuse == "cemetery" or object.tags.natural == "wood" or
            object.tags.natural == "water" or object.tags.leisure == "park" or object.tags.landuse == "residential") then
        landcover_far:insert({
            name = object.tags.name,
            geom = object:as_multipolygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            landcover = object.tags.landcover
        })
    end
end

function osm2pgsql.process_node(object)
    if (object.tags.place) then
        all_node:insert({
            name = object.tags.name,
            geom = object:as_point(),
            tags = object.tags,
            place = object.tags.place
        })
    end

    if object.tags.place == "ocean" or object.tags.place == "sea" then
        water_name:insert({
            name = object.tags.name,
            geom = object:as_point(),
            tags = object.tags,
            place = object.tags.place
        })
    end
end
