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
}})

local cycleway_point = osm2pgsql.define_node_table('cycleway_node', {{
    column = 'name',
    type = 'text'
}, {
    column = 'geom',
    type = 'Point'
}, {
    column = 'tags',
    type = 'jsonb'
}, {
    column = 'adjacents',
    sql_type = 'int8[]'
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

    if object.tags.highway and object.tags.highway ~= 'motorway' and object.tags.bicycle ~= 'no' and object.tags.highway then

        all_way:insert({
            name = object.tags.name,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            tags = object.tags,
            nodes = "{" .. table.concat(object.nodes, ",") .. "}"
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
