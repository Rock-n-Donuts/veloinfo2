if ("serviceWorker" in navigator) {
    navigator.serviceWorker.register("/pub/service-worker.js");
}

(async () => {
    try {
        const wakeLock = await navigator.wakeLock.request("screen");
    } catch (err) {
        // the wake lock request fails - usually system related, such being low on battery
        console.log(`${err.name}, ${err.message}`);
    }
})();

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
        enableHighAccuracy: false
    },
    trackUserLocation: true
}));

map.on("load", () => {
    clear();
})


const state = {
    mode: "select"
};
map.on("click", async function (event) {
    if (state.mode == "select") {
        select(event);
    }
});

map.on("move", function (e) {
    document.cookie = "zoom=" + map.getZoom();
    document.cookie = "lng=" + map.getCenter().lng;
    document.cookie = "lat=" + map.getCenter().lat;

    update_url();
});

let start_marker = null;
let end_marker = null;
async function select(event) {
    if (start_marker) {
        start_marker.remove();
    }
    start_marker = new maplibregl.Marker({ color: "#00f" }).setLngLat([event.lngLat.lng, event.lngLat.lat]).addTo(map);

    let width = 20;
    var features = map.queryRenderedFeatures(
        [
            [event.point.x - width / 2, event.point.y - width / 2],
            [event.point.x + width / 2, event.point.y + width / 2]
        ], { layers: ['cycleway', "designated", "shared_lane"] });

    if (features.length) {
        var feature = features[0];
        var point = await fetch('/select/node/' + event.lngLat.lng + "/" + event.lngLat.lat);
        var point = await point.json();
        way_ids = point.way_id;
        display_segment_geom([point.geom]);

        // Display info panel
        var info_panel = document.getElementById("info");
        const response = await fetch("/segment_panel/ways/" + way_ids);
        const html = await response.text();
        info_panel.innerHTML = html;
        // reprocess htmx for the new info panel
        info_panel = document.getElementById("info");
        htmx.process(info_panel);
    } else {
        clear();

    }

    if (!start_marker) {
    } else {
        // var point = await fetch('/select/node/' + event.lngLat.lng + "/" + event.lngLat.lat);
        // var point = await point.json();
        // if (end_marker) {
        //     end_marker.remove();
        // }
        // end_marker = new maplibregl.Marker({ color: "#f00" }).setLngLat([point.lng, point.lat]).addTo(map);

        // var nodes = await fetch('/route/' + start_marker.getLngLat().lng + "/" + start_marker.getLngLat().lat + "/" + event.lngLat.lng + "/" + event.lngLat.lat);
        // var nodes = await nodes.json();
        // console.log("nodes: ", nodes);
        // var route = nodes.map((coords) => {
        //     return [coords.x, coords.y];
        // });
        // way_ids = nodes.map((node) => node.way_id);
        // way_ids = [...new Set(way_ids)].join(" ");

        // display_segment_geom([route]);
    }

    if (way_ids) {
    }
}

async function zoomToSegment(score_id) {
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
    // find the largest bounds
    var bounds = geom.reduce((currentBounds, coord) => {
        return [
            [Math.min(coord[0], currentBounds[0][0]), Math.min(coord[1], currentBounds[0][1])], // min coordinates
            [Math.max(coord[0], currentBounds[1][0]), Math.max(coord[1], currentBounds[1][1])]  // max coordinates
        ];
    }, [[Infinity, Infinity], [-Infinity, -Infinity]]);
    map.fitBounds(bounds, { padding: window.innerWidth * .10 });
}

function display_segment_geom(geom) {
    console.log("geom: ", geom);
    if (map.getLayer("selected")) {
        console.log("updating selected layer");
        map.getSource("selected").setData({
            "type": "Feature",
            "properties": {},
            "geometry": {
                "type": "MultiLineString",
                "coordinates": geom
            }
        });
    } else {
        map.addSource("selected", {
            "type": "geojson",
            "data": {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "MultiLineString",
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
                "line-color": "#800",
                "line-opacity": 0.3
            }
        });
    }

}

let timeout_info = null;
async function update_info() {
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
function update_url() {
    if (timeout_url) {
        clearTimeout(timeout_url);
    }
    timeout_url = setTimeout(() => {
        window.history.replaceState({}, "", "/?lat=" + map.getCenter().lat + "&lng=" + map.getCenter().lng + "&zoom=" + map.getZoom());
        update_info();
    }, 1000);
}


async function clear() {
    state.mode = "select";
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

async function reset() {
    map.getSource("veloinfo").setUrl("{{martin_url}}/bike_path");
}


function getCookie(name) {
    let matches = document.cookie.match(new RegExp(
        "(?:^|; )" + name.replace(/([\.$?*|{}\(\)\[\]\\\/\+^])/g, '\\$1') + "=([^;]*)"
    ));
    return matches ? decodeURIComponent(matches[1]) : undefined;
}

async function route() {
    const button = document.getElementById("route_button");
    button.classList.add("htmx-request");
    var end = start_marker.getLngLat();
    // get the position of the device
    var start = await new Promise((resolve, reject) => {
        navigator.geolocation.getCurrentPosition((position) => {
            resolve(position);
        });
    });
    await htmx.ajax("GET", "/route/" + start.coords.longitude + "/" + start.coords.latitude + "/" + end.lng + "/" + end.lat, "#info");
}