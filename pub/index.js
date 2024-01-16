var response = { geom: null, source: null, target: null, node: null };
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

    fetch_response = await fetch('/cycleway/' + feature.properties.way_id, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(response)
    });
    response = await fetch_response.json();

    if (map.getLayer("selected")) {
        map.getSource("selected").setData({
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "LineString",
                "coordinates": response.geom
            }
        })
    } else {
        map.addSource("selected", {
            "type": "geojson",
            "data": {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "LineString",
                    "coordinates": response.geom
                }
            }
        })
        map.addLayer({
            "id": "selected",
            "type": "line",
            "source": "selected",
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
    }

});