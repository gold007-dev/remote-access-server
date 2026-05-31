FROM scratch

COPY ./ras-rs/target/release/ras-rs /ras-rs

CMD ["/ras-rs"]
