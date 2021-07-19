FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/foo /usr/local/bin/
COPY ./target/x86_64-unknown-linux-musl/release/bar /usr/local/bin/
USER 1001
EXPOSE 8080
ENTRYPOINT ["foo"]
