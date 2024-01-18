local cycleway = osm2pgsql.define_way_table("cycleway", { {
    column = 'name',
    type = 'text'
}, {
    column = 'winter_service',
    type = 'text'
}, {
    column = 'geom',
    type = 'LineString'
}, {
    column = 'source',
    type = 'int8'
}, {
    column = 'target',
    type = 'int8'
},{
    column = 'kind',
    type = 'text'} 

})

function osm2pgsql.process_way(object)
    if object.tags.highway == 'cycleway' or object.tags.cycleway == "track" or object.tags["cycleway:left"] == "track" or
        object.tags["cycleway:right"] == "track" or object.tags["cycleway:both"] == "track" then
        
        cycleway:insert({
            name = object.tags.name,
            winter_service = object.tags.winter_service,
            geom = object:as_linestring(),
            source = object.nodes[1],
            target = object.nodes[#object.nodes],
            kind = 'cycleway'
        })
    elseif object.tags.bicycle == "designated" or object.tags["cycleway:left"] == "share_busway" or
        object.tags["cycleway:right"] == "share_busway" or object.tags["cycleway:both"] == "share_busway" or
        object.tags["cycleway:right"] == "lane" or object.tags["cycleway:left"] == "lane" or
        object.tags["cycleway:both"] == "lane" then
            cycleway:insert({
                name = object.tags.name,
                winter_service = object.tags.winter_service,
                geom = object:as_linestring(),
                source = object.nodes[1],
                target = object.nodes[#object.nodes],

                kind = 'designated'
    })
    elseif object.tags.cycleway == "shared_lane" or object.tags.cycleway == "lane" or object.tags["cycleway:left"] == "shared_lane" or object.tags["cycleway:right"] == "shared_lane" or object.tags["cycleway:both"] == "shared_lane" then
            cycleway:insert({
                name = object.tags.name,
                winter_service = object.tags.winter_service,
                geom = object:as_linestring(),
                source = object.nodes[1],
                target = object.nodes[#object.nodes],
                kind = 'shared_lane'
            })
    end
end
