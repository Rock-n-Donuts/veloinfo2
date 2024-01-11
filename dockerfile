FROM rust

RUN apt-get update && apt-get install -y \
    fish \
    rustfmt \
    osm2pgsql \
    nodejs npm  
    
RUN chsh -s $(which fish)
RUN cargo install cargo-watch
RUN rustup component add rustfmt

RUN install -d tailwindcss
RUN echo "db:5432:carte:postgres:postgres" >> /root/.pgpass
RUN chmod 0600 /root/.pgpass

WORKDIR /app
CMD cargo watch -x run
