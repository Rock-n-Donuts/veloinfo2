use crate::db::cycleway::Cycleway;
use crate::{VeloInfoError, VeloinfoState};
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn select(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
) -> Result<Json<Cycleway>, VeloInfoError> {
    let conn = state.conn;
    let searched_segment: Cycleway = Cycleway::get(&way_id, conn.clone()).await?;

    Ok(Json(searched_segment))
}

// pub async fn route(
//     State(state): State<VeloinfoState>,
//     Path((way_id1, way_ids)): Path<(i64, String)>,
// ) -> Result<Json<Route>, VeloInfoError> {
//     let re = Regex::new(r"\d+")?;
//     let conn = state.conn;
//     let way_ids_i64 = re
//         .find_iter(&way_ids)
//         .map(|m| m.as_str().parse::<i64>().unwrap_or_default())
//         .collect::<Vec<i64>>();
//     let start_segment: Cycleway = Cycleway::get(&way_id1, conn.clone()).await?;
//     let cycleways: Vec<Cycleway> = join_all(
//         way_ids_i64
//             .iter()
//             .map(|way_id| async { Cycleway::get(&way_id.clone(), conn.clone()).await }),
//     )
//     .await
//     .iter().filter_map(|c| match c {
//         Ok(c) => Some(c),
//         Err(_) => None,
//     }).cloned().collect();

//     let mut routes: Vec<Route> = vec![];
//     // We try to find the longest path between the 4 possible combinations
//     // It is not the best way to do it, but it is the simplest
//     routes.push(
//         Cycleway::route(
//             &start_segment.source,
//             &cycleways.first().expect("uncomplete cycleways").target,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.source,
//             &cycleways.first().expect("uncomplete cycleways").source,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.source,
//             &cycleways.last().expect("uncomplete cycleways").target,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.source,
//             &cycleways.last().expect("uncomplete cycleways").source,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.target,
//             &cycleways.first().expect("uncomplete cycleways").target,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.target,
//             &cycleways.first().expect("uncomplete cycleways").source,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.target,
//             &cycleways.last().expect("uncomplete cycleways").target,
//             conn.clone(),
//         )
//         .await?,
//     );
//     routes.push(
//         Cycleway::route(
//             &start_segment.target,
//             &cycleways.last().expect("uncomplete cycleways").source,
//             conn.clone(),
//         )
//         .await?,
//     );

//     // We keep the longest segment
//     let merge = routes
//         .iter()
//         .max_by(|x, y| x.geom.len().cmp(&y.geom.len()))
//         .expect("no bigger segment")
//         .to_owned();
//     Ok(Json(merge))
// }
