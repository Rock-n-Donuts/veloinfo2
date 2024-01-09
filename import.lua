local cycleway = osm2pgsql.define_way_table("cycleway", {
    {
        column = 'name',
        type = 'text'
    }, {
        column = 'winter_service',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
    }
})

local shared = osm2pgsql.define_way_table("shared", {
    {
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
    }
})

local designated = osm2pgsql.define_way_table("designated", {
    {
        column = 'name',
        type = 'text'
    }, {
        column = 'geom',
        type = 'LineString',
    }
})

function osm2pgsql.process_way(object)
    if object.tags.highway == 'cycleway' or 
        object.tags.cycleway == "track" then
        cycleway:insert({
            name = object.tags.name,
            winter_service = object.tags.winter_service,
            geom = object:as_linestring()
        })
    end

    if object.tags.bicycle == "designated" or
        object.tags["cycleway:left"] == "share_busway" or 
        object.tags["cycleway:right"] == "share_busway" or
        object.tags["cycleway:both"] == "share_busway" or
        object.tags["cycleway:left"] == "shared_lane" or 
        object.tags["cycleway:right"] == "shared_lane" or
        object.tags["cycleway:both"] == "shared_lane" or
        object.tags["cycleway:right"] == "lane" or
        object.tags["cycleway:left"] == "lane" or
        object.tags["cycleway:both"] == "lane" then
        designated:insert({
            name = object.tags.name,
            geom = object:as_linestring()
        })
    end

    if object.tags.cycleway == "shared_lane" or 
        object.tags.cycleway == "lane" then
        shared:insert({
            name = object.tags.name,
            geom = object:as_linestring()
        })
    end
end

