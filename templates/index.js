getCookie = (name) => {
    let matches = document.cookie.match(new RegExp(
        "(?:^|; )" + name.replace(/([\.$?*|{}\(\)\[\]\\\/\+^])/g, '\\$1') + "=([^;]*)"
    ));
    return matches ? decodeURIComponent(matches[1]) : undefined;
}

if ("serviceWorker" in navigator) {
    navigator.serviceWorker.register("/pub/service-worker.js");
}
var way_ids = "";

// Set the initial map center and zoom level
// the url parameters take precedence over the cookies
var lng = getCookie("lng") ? getCookie("lng") : -72.45272261855519;
var lat = getCookie("lat") ? getCookie("lat") : 45.924806212523265;
var zoom = getCookie("zoom") ? getCookie("zoom") : 6;
let params = new URLSearchParams(window.location.search);
if (params.has("lat") && params.has("lng") && params.has("zoom")) {
    lat = parseFloat(params.get("lat"));
    lng = parseFloat(params.get("lng"));
    zoom = parseFloat(params.get("zoom"));
}

var map = new maplibregl.Map({
    container: 'map',
    style: '/style.json',
    center: [lng, lat],
    zoom: zoom
});

map.addControl(new maplibregl.NavigationControl());
map.addControl(new maplibregl.GeolocateControl({
    positionOptions: {
        enableHighAccuracy: true
    },
    trackUserLocation: true
}));

map.on("load", () => {
    clear();
})


map.on("click", async function (event) {
    if (map.getZoom() > 15) {
        select(event);
    }
});

map.on("move", function (e) {
    document.cookie = "zoom=" + map.getZoom();
    document.cookie = "lng=" + map.getCenter().lng;
    document.cookie = "lat=" + map.getCenter().lat;

    update_url();
});

select = async (event) => {
    let width = 40;
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


    var fetch_response = await fetch('/segment/select/' + feature.properties.way_id);
    var response = await fetch_response.json();

    const segment_panel = document.getElementById("segment_panel");
    if (segment_panel) {
        fetch_response = await fetch('/segment/route/' + feature.properties.way_id + "/" + way_ids);
        response = await fetch_response.json();
        if (response.way_ids.length == 0) {
            return;
        }
        way_ids = response.way_ids;
    } else {
        way_ids = feature.properties.way_id;
    }
    display_segment_geom(response.geom);
    if (way_ids) {
        // Display info panel
        var info_panel = document.getElementById("info");
        const response = await fetch("/segment_panel/ways/" + way_ids);
        const html = await response.text();
        info_panel.innerHTML = html;
        // reprocess htmx for the new info panel
        info_panel = document.getElementById("info");
        htmx.process(info_panel);
    }

}

zoomToSegment = async (score_id) => {
    var fetch_response = await fetch('/cyclability_score/geom/' + score_id);
    var response = await fetch_response.json();
    way_ids = response.reduce((way_ids, score) => {
        return way_ids + " " + score.way_id;
    }, "");
    var geom = response.reduce((geom, cycleway) => {
        cycleway.geom.forEach((coords) => {
            geom.push(coords);
        });
        return geom;
    }, []);
    display_segment_geom(geom);
}

display_segment_geom = async (geom) => {
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

    // find the largest bounds
    var bounds = geom.reduce((currentBounds, coord) => {
        return [
            [Math.min(coord[0], currentBounds[0][0]), Math.min(coord[1], currentBounds[0][1])], // min coordinates
            [Math.max(coord[0], currentBounds[1][0]), Math.max(coord[1], currentBounds[1][1])]  // max coordinates
        ];
    }, [[Infinity, Infinity], [-Infinity, -Infinity]]);
    map.fitBounds(bounds, { padding: window.innerWidth * .10 });
}

let timeout_info = null;
update_info = async () => {
    if (timeout_info) {
        clearTimeout(timeout_info);
    }
    timeout_info = setTimeout(async () => {
        var info_panel = document.getElementById("info_panel_up");
        if (!info_panel) {
            return;
        }
        clear();
    }, 1000)
}

let timeout_url = null;
const update_url = () => {
    if (timeout_url) {
        clearTimeout(timeout_url);
    }
    timeout_url = setTimeout(() => {
        window.history.replaceState({}, "", "/?lat=" + map.getCenter().lat + "&lng=" + map.getCenter().lng + "&zoom=" + map.getZoom());
        update_info();
    }, 1000);
}


clear = async () => {
    const selected = map.getSource("selected");
    if (selected) {
        selected.setData({
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "LineString",
                "coordinates": []
            }
        });
    }
    // Display info panel
    var segment_panel = document.getElementById("info");
    var hx_indicator = document.getElementsByClassName("htmx-indicator")[0];
    if (hx_indicator) {
        hx_indicator.classList.add("htmx-request");
    }
    const response = await fetch("/info_panel/up", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(map.getBounds())
    });
    var hx_indicator = document.getElementsByClassName("htmx-indicator")[0];
    if (hx_indicator) {
        hx_indicator.classList.remove("htmx-request");
    }
    const html = await response.text();
    segment_panel.innerHTML = html;
    // reprocess htmx for the new info panel
    segment_panel = document.getElementById("info");
    htmx.process(segment_panel);
}

reset = async () => {
    map.getSource("veloinfo").setUrl("{{martin_url}}/bike_path");
}



