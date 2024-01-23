var response = { geom: null, source: null, target: null, node: null };
var way_ids = "";
var map = new maplibregl.Map({
    container: 'map',
    style: 'http://localhost:3000/pub/style.json',
    center: [getCookie("lng") ? getCookie("lng") : -72.45272261855519, getCookie("lat") ? getCookie("lat") : 45.924806212523265],
    zoom: getCookie("zoom") ? getCookie("zoom") : 6
});

map.on("click", async function (event) {
    select(event);
});

map.on("move", function (e) {
    document.cookie = "zoom=" + map.getZoom();
    document.cookie = "lng=" + map.getCenter().lng;
    document.cookie = "lat=" + map.getCenter().lat;
});

select = async (event) => {
    let width = 20;
    var features = map.queryRenderedFeatures(
        [
            [event.point.x - width / 2, event.point.y - width / 2],
            [event.point.x + width / 2, event.point.y + width / 2]
        ], { layers: ['cycleway', "designated", "shared_lane"] });
    if (!features.length) {
        clear();
        return;
    }
    var feature = features[0];

    var fetch_response;
    const info_panel = document.getElementById("select_score");

    var fetch_response = await fetch('/segment/select/' + feature.properties.way_id);
    
    var response = await fetch_response.json();
    if(info_panel) {
        fetch_response = await fetch('/segment/route/' + feature.properties.way_id + "/" +way_ids);
        response = await fetch_response.json();
        way_ids = response.way_ids;
    }else{
        way_ids = feature.properties.way_id;
    }
    display_segment(response.geom, response.way_id);
}

display_segment = async (geom, way_id) => {
    if (map.getLayer("selected")) {
        map.getSource("selected").setData({
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "LineString",
                "coordinates": geom
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
                    "coordinates": geom
                }
            }
        })
        map.addLayer({
            "id": "selected",
            "type": "line",
            "source": "selected",
            "paint": {
                "line-width": 12,
                "line-color": "#000",
                "line-opacity": 0.3
            }
        });
    }

    // Display info panel
    var info_panel = document.getElementById("info_panel");
    const response = await fetch("/info_panel/" + way_ids);
    const html = await response.text();
    info_panel.outerHTML = html;
    // reprocess htmx for the new info panel
    info_panel = document.getElementById("info_panel");
    htmx.process(info_panel);
}

clear = async () => {
    way_ids = "";
    map.getSource("selected").setData({
        "type": "Feature",
        "properties": {},
        "geometry": {
            "type": "LineString",
            "coordinates": []
        }
    })
    // Display info panel
    var info_panel = document.getElementById("info_panel");
    const response = await fetch("/info_panel");
    const html = await response.text();
    info_panel.outerHTML = html;
    // reprocess htmx for the new info panel
    info_panel = document.getElementById("info_panel");
    htmx.process(info_panel);
}

reset = async () => {
    map.getSource("veloinfo").setUrl("http://localhost:3001/bike_path");
}

function getCookie(name) {
    let matches = document.cookie.match(new RegExp(
        "(?:^|; )" + name.replace(/([\.$?*|{}\(\)\[\]\\\/\+^])/g, '\\$1') + "=([^;]*)"
    ));
    return matches ? decodeURIComponent(matches[1]) : undefined;
}