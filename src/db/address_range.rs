use serde::Serialize;
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
            r#"SELECT city, street, lng, lat FROM (
                SELECT city, street, ST_X(st_transform(ST_PointN(geom, 1), 4326)) as lng, ST_Y(st_transform(ST_PointN(geom, 1), 4326)) as lat,
                ROW_NUMBER() OVER(PARTITION BY city, street ORDER BY ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($2, $3), 4326), 3857)) AS rn
                FROM address_range ar 
                WHERE tsvector  @@ websearch_to_tsquery('french', $1)
                order by ar.geom<-> ST_Transform(ST_SetSRID(ST_MakePoint($2, $3), 4326), 3857)
            ) t WHERE rn = 1
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
    conn: &sqlx::Pool<Postgres>,
) -> Vec<AddressRange> {
    println!("get_with_adress: {:?}", number);
    println!("get_with_adress: {:?}", request);
    let odd_even = if number % 2 == 0 { "even" } else { "odd" };
    let r = match sqlx::query_as(
        r#"select city, 
                street, 
                ST_X(st_transform(ST_PointN(geom, 1), 4326)) as lng, 
                ST_Y(st_transform(ST_PointN(geom, 1), 4326)) as lat
            from address_range ar 
            where tsvector  @@ websearch_to_tsquery('french', $1) and
            (start <= $2 and "end" >= $2 or start >= $2 and "end" <= $2 )and
            odd_even = $3
            "#,
    )
    .bind(request)
    .bind(number)
    .bind(odd_even)
    .fetch_all(conn)
    .await
    {
        Ok(ar) => ar,
        Err(e) => {
            eprintln!("Error getting address_range: {}", e);
            Vec::new()
        }
    };
    println!("get_with_adress: {:?}", r);
    r
}
