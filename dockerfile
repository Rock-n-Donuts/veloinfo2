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

WORKDIR /app
CMD cargo watch -x run
