use sqlx::Postgres;

#[derive(sqlx::FromRow, Debug)]
pub struct AddressRange {
    pub city: String,
    pub street: String,
    pub lng: f64,
    pub lat: f64,
}

impl AddressRange {
    pub async fn get(
        request: &String,
        lng: &f64,
        lat: &f64,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<AddressRange> {
        match sqlx::query_as(
            r#"select city, street, ST_X(st_transform(ST_PointN(geom, 1), 4326)) as lng, ST_Y(st_transform(ST_PointN(geom, 1), 4326)) as lat
                    from address_range ar 
                    where tsvector  @@ websearch_to_tsquery('french', $1)
                    order by ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($2, $3), 4326), 3857)
                    limit 40
            "#,
        )
        .bind(request)
        .bind(lng)
        .bind(lat)
        .fetch_all(conn)
        .await
        {
            Ok(ar) => ar,
            Err(e) => {
                eprintln!("Error getting address_range: {}", e);
                Vec::new()
            }
        }
    }
}
