var layers_showned = new Set();
var map = new maplibregl.Map({
    container: 'map',
    style: 'http://localhost:3000/pub/style.json'
});

map.on("click", async function (e) {
    let width = 20;
    var features = map.queryRenderedFeatures(
        [
            [e.point.x - width / 2, e.point.y - width / 2],
            [e.point.x + width / 2, e.point.y + width / 2]
        ], { layers: ['cycleway'] });
    if (!features.length) {
        return;
    }
    var feature = features[0];

    if (layers_showned.has(feature.properties.way_id)) {
        map.removeLayer(feature.properties.way_id + "");
        map.removeSource(feature.properties.way_id + "");
        layers_showned.delete(feature.properties.way_id);
        return;
    }
    layers_showned.add(feature.properties.way_id);

    response = await fetch('/cycleway/' + feature.properties.way_id);
    response = (await response.json());
    map.addSource(response.way_id + "", {
        "type": "geojson",
        "data": {
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "LineString",
                "coordinates": response.line_string.points
            }
        }
    })
    map.addLayer({
        "id": response.way_id + "",
        "type": "line",
        "source": response.way_id + "",
        "layout": {
            "line-join": "round",
            "line-cap": "round"
        },
        "paint": {
            "line-width": 10,
            "line-color": "#f00",
            "line-blur": 10,
        }
    });
});