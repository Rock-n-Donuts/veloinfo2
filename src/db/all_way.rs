use sqlx::Postgres;

use super::cycleway::{Node, NodeDb};

async fn find_closest_node(
    lng: f64,
    lat: f64,
    conn: sqlx::Pool<Postgres>,
) -> Result<Node, sqlx::Error> {
    let response: NodeDb = match sqlx::query_as(
        r#"        
        SELECT
            way_id,
            geom,
            unnest(nodes) as node_id,
            ST_X(st_transform((dp).geom, 4326)) as lng,
            ST_Y(st_transform((dp).geom, 4326)) as lat
        FROM (  
            SELECT (ST_DumpPoints(geom)) as dp, 
                    way_id,
                    ST_AsText(ST_Transform(geom, 4326)) as geom,
                    nodes
            FROM all_way
            WHERE ST_DWithin(geom, ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857), 1000)
        ) as subquery
        ORDER BY (dp).geom <-> ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857)
        LIMIT 1"#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_one(&conn)
    .await
    {
        Ok(response) => response,
        Err(e) => return Err(e),
    };
    Ok(response.into())
}
