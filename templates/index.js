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
    zoom: zoom,
    minZoom: 8
});

map.addControl(new maplibregl.NavigationControl());
map.addControl(new maplibregl.GeolocateControl({
    positionOptions: {
        enableHighAccuracy: false
    },
    trackUserLocation: true
}));

map.on("load", () => {
    const bounds = map.getBounds();
    htmx.ajax("GET", "/info_panel/up/" + bounds._sw.lng + "/" + bounds._sw.lat + "/" + bounds._ne.lng + "/" + bounds._ne.lat, "#info");
})


map.on("click", async function (event) {
    if (document.getElementById("info_panel_up") ||
        document.getElementById("info_panel_down") ||
        document.getElementById("segment_panel_bigger") ||
        document.getElementById("segment_panel")) {
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
    const segment_panel_bigger = document.getElementById("segment_panel_bigger");
    if (segment_panel_bigger) {
        selectBigger(event);
        return;
    }

    if (start_marker) {
        start_marker.remove();
    }
    start_marker = new maplibregl.Marker({ color: "#00f" }).setLngLat([event.lngLat.lng, event.lngLat.lat]).addTo(map);

    let width = 20;
    var features = map.queryRenderedFeatures(
        [
            [event.point.x - width / 2, event.point.y - width / 2],
            [event.point.x + width / 2, event.point.y + width / 2]
        ], { layers: ['cycleway-zoom'] });

    if (features.length) {
        var feature = features[0];
        htmx.ajax('GET', '/segment_panel_lng_lat/' + event.lngLat.lng + "/" + event.lngLat.lat, "#info");
    } else {
        clear();
    }
}

async function selectBigger(event) {
    if (end_marker) {
        end_marker.remove();
    }
    end_marker = new maplibregl.Marker({ color: "#f00" }).setLngLat([event.lngLat.lng, event.lngLat.lat]).addTo(map);

    var nodes = await htmx.ajax('GET', '/segment_panel_bigger/' + start_marker.getLngLat().lng + "/" + start_marker.getLngLat().lat + "/" + event.lngLat.lng + "/" + event.lngLat.lat, "#info");
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
}

function fitBounds(geom) {
    console.log("fitBounds", geom);
    var bounds = geom.reduce((currentBounds, coord) => {
        return [
            [Math.min(coord[0], currentBounds[0][0]), Math.min(coord[1], currentBounds[0][1])], // min coordinates
            [Math.max(coord[0], currentBounds[1][0]), Math.max(coord[1], currentBounds[1][1])]  // max coordinates
        ];
    }, [[Infinity, Infinity], [-Infinity, -Infinity]]);
    console.log("bounds", bounds);
    return map.cameraForBounds(bounds, { padding: window.innerWidth * .10 });
}

function display_segment_geom(geom) {
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
    if (!start_marker) {
        start_marker = new maplibregl.Marker({ color: "#00f" }).setLngLat(geom[0][0]).addTo(map);
    }
    map.getSource("veloinfo").setUrl("{{martin_url}}/bike_path");
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
    if (start_marker) {
        start_marker.remove();
    }
    if (end_marker) {
        end_marker.remove();
    }
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
    const bounds = map.getBounds();
    htmx.ajax("GET", "/info_panel/up/" + bounds._sw.lng + "/" + bounds._sw.lat + "/" + bounds._ne.lng + "/" + bounds._ne.lat, "#info");
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
function calculateBearing(lon1, lat1, lon2, lat2) {
    lon1 = lon1 * Math.PI / 180.0;
    lat1 = lat1 * Math.PI / 180.0;
    lon2 = lon2 * Math.PI / 180.0;
    lat2 = lat2 * Math.PI / 180.0;
    const y = Math.sin(lon2 - lon1) * Math.cos(lat2);
    const x = Math.cos(lat1) * Math.sin(lat2) - Math.sin(lat1) * Math.cos(lat2) * Math.cos(lon2 - lon1);
    let bearing = Math.atan2(y, x) * (180 / Math.PI);
    bearing = (bearing + 360) % 360; // Ensuring the bearing is positive
    return bearing;
}