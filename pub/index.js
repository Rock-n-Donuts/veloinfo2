var response = { geom: null, source: null, target: null, node: null };
var map = new maplibregl.Map({
    container: 'map',
    style: 'http://localhost:3000/pub/style.json',
    center: [getCookie("lng") ? getCookie("lng") : -72.45272261855519, getCookie("lat") ? getCookie("lat") : 45.924806212523265],
    zoom: getCookie("zoom") ? getCookie("zoom") : 6
});

map.on("click", async function (e) {
    select(e);
});

map.on("move", function (e) {
    document.cookie = "zoom=" + map.getZoom();
    document.cookie = "lng=" + map.getCenter().lng;
    document.cookie = "lat=" + map.getCenter().lat;
});

select = async (e) => {
    let width = 20;
    var features = map.queryRenderedFeatures(
        [
            [e.point.x - width / 2, e.point.y - width / 2],
            [e.point.x + width / 2, e.point.y + width / 2]
        ], { layers: ['cycleway', "designated", "shared_lane"] });
    if (!features.length) {
        return;
    }
    var feature = features[0];

    fetch_response = await fetch('/cycleway/select/' + feature.properties.way_id);
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
                "line-width": 8,
                "line-color": "#f00",
            }
        });
    }
    buttons = document.getElementById("edit_buttons");
    const event = new CustomEvent('selected', { bubbles: true, detail: response });
    buttons.dispatchEvent(event);

}

function getCookie(name) {
    let matches = document.cookie.match(new RegExp(
        "(?:^|; )" + name.replace(/([\.$?*|{}\(\)\[\]\\\/\+^])/g, '\\$1') + "=([^;]*)"
    ));
    return matches ? decodeURIComponent(matches[1]) : undefined;
}