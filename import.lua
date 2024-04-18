-- Imports OSM tags into PGSQL database

local cycleway = osm2pgsql.define_way_table("cycleway_way", { {
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
} })

local all_way = osm2pgsql.define_way_table("all_way", {
    {
        column = 'name',
        type = 'text'
    },
    {
        column = 'geom',
        type = 'LineString',
        not_null = true
    },
    {
        column = 'source',
        type = 'int8',
        not_null = true
    },
    {
        column = 'target',
        type = 'int8',
        not_null = true
    },
    {
        column = 'tags',
        type = 'jsonb',
        not_null = true
    },
    {
        column = 'nodes',
        sql_type = 'int8[] NOT NULL'
    },
    {
        column = 'landuse',
        type = 'text'
    },
    {
        column = 'tunnel',
        type = 'text'
    },
    {
        column = 'bridge',
        type = 'text'
    }
})

local all_area = osm2pgsql.define_table({
    name = 'all_area',
    ids = {
        type = 'area',
        id_column = 'way_id'
    },
    columns = { {
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
        column = 'aeroway',
        type = 'text'
    }, {
        column = 'man_made',
        type = 'text'
    }

    },
    indexes = { {
        column = 'geom',
        method = 'gist'
    } }

})

local cycleway_point = osm2pgsql.define_node_table('cycleway_point', { {
    column = 'name',
    type = 'text'
}, {
    column = 'geom',
    type = 'Point'
}, {
    column = 'tags',
    type = 'jsonb'
} })

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

    if object.tags.tunnel or object.tags.highway or object.tags.bridge then
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

    if object.is_closed and (object.tags.natural or object.tags.landuse or object.tags.leisure or object.tags.aeroway or object.tags.man_made) then
        all_area:insert({
            name = object.tags.name,
            geom = object:as_polygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            aeroway = object.tags.aeroway,
            man_made = object.tags.man_made
        })
    end
end

function osm2pgsql.process_relation(object)
    if object.tags.natural or object.tags.landuse or object.tags.leisure or object.tags.aeroway or object.tags.man_made then
        all_area:insert({
            name = object.tags.name,
            geom = object:as_multipolygon(),
            tags = object.tags,
            landuse = object.tags.landuse,
            natural = object.tags.natural,
            leisure = object.tags.leisure,
            aeroway = object.tags.aeroway,
            man_made = object.tags.man_made
        })
    end
end

function osm2pgsql.process_node(object)
    cycleway_point:insert({
        name = object.tags.name,
        geom = object:as_point(),
        tags = object.tags
    })
end
