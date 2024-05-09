use sqlx::Postgres;

#[derive(sqlx::FromRow, Debug)]
pub struct AddressRange {
    pub city: String,
    pub street: String,
    pub lng: f64,
    pub lat: f64,
}

pub async fn get(
    request: &String,
    lng: &f64,
    lat: &f64,
    conn: &sqlx::Pool<Postgres>,
) -> Vec<AddressRange> {
    match sqlx::query_as(
            r#"SELECT COALESCE(city,'') as city, street, lng, lat FROM (
                SELECT city, 
                    street, 
                    CASE
                        WHEN ST_GeometryType(geom) = 'ST_Point' THEN ST_X(ST_Transform(geom, 4326))
                        ELSE ST_X(ST_Transform(ST_PointN(geom, 1), 4326))
                    END as lng,
                    CASE
                        WHEN ST_GeometryType(geom) = 'ST_Point' THEN ST_Y(ST_Transform(geom, 4326))
                        ELSE ST_Y(ST_Transform(ST_PointN(geom, 1), 4326))
                    END as lat,
                    ROW_NUMBER() OVER(PARTITION BY city, street ORDER BY ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($2, $3), 4326), 3857)) AS rn
                FROM address_range ar 
                WHERE tsvector  @@ websearch_to_tsquery('french', $1)
                order by ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($2, $3), 4326), 3857)
            ) t WHERE rn = 1
            limit 20
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

pub async fn get_with_adress(
    number: &i64,
    request: &String,
    lng: &f64,
    lat: &f64,
    conn: &sqlx::Pool<Postgres>,
) -> Vec<AddressRange> {
    println!("get_with_adress: {:?}", number);
    println!("get_with_adress: {:?}", request);
    let odd_even = if number % 2 == 0 { "even" } else { "odd" };
    let r = match sqlx::query_as(
        r#"select 
                COALESCE(city,'') as city, 
                street, 
                CASE
                    WHEN ST_GeometryType(geom) = 'ST_Point' THEN ST_X(ST_Transform(geom, 4326))
                    ELSE ST_X(ST_Transform(ST_PointN(geom, 1), 4326))
                END as lng,
                CASE
                    WHEN ST_GeometryType(geom) = 'ST_Point' THEN ST_Y(ST_Transform(geom, 4326))
                    ELSE ST_Y(ST_Transform(ST_PointN(geom, 1), 4326))
                END as lat
            from address_range ar 
            where tsvector  @@ websearch_to_tsquery('french', $1) and
                (start <= $2 and "end" >= $2 or start >= $2 and "end" <= $2 )and
                odd_even = $3
            order by ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($4, $5), 4326), 3857)"#,
    )
    .bind(request)
    .bind(number)
    .bind(odd_even)
    .bind(lng)
    .bind(lat)
    .fetch_all(conn)
    .await
    {
        Ok(ar) => ar,
        Err(e) => {
            eprintln!("Error getting address_range 2: {}", e);
            Vec::new()
        }
    };
    println!("get_with_adress: {:?}", r);
    r
}
