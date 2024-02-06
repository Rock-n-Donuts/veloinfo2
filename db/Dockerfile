FROM postgres:13

LABEL maintainer="PostGIS Project - https://postgis.net"

# ENV POSTGIS_MAJOR 3
# ENV POSTGIS_VERSION 3.2.1+dfsg-1.pgdg110+1

RUN apt-get update \
      && apt-get install -y \
           postgresql-13-postgis-3 \
           postgresql-13-pgrouting

RUN ln -s /usr/lib/postgresql/13/lib/libpgrouting-3.6.so /usr/lib/postgresql/13/lib/libpgrouting-3.2.so 

RUN mkdir -p /docker-entrypoint-initdb.d
COPY ./initdb-postgis.sh /docker-entrypoint-initdb.d/10_postgis.sh
COPY ./update-postgis.sh /usr/local/bin