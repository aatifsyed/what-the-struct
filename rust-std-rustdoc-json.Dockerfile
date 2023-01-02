FROM rust

ARG target=x86_64-unknown-linux-gnu
ARG commit=50a8ca56be293d34d4754783ba15eb5fb4e20619
# commit 50a8ca56be293d34d4754783ba15eb5fb4e20619
# Author: Lukas Markeffsky <@>
# Date:   Mon Dec 19 15:03:04 2022 +0100
# 
#     `./x doc library --open` opens `std`

WORKDIR /rust-lang-rust

# Note: can't simply used the released tarball on the tag, as it's missing some directories

RUN : \
    && git init \
    && git remote add origin https://github.com/rust-lang/rust \
    && git fetch origin "${commit}" \
    && git reset --hard FETCH_HEAD

RUN : \
    && mkdir /update-crates-io \
    && cd /update-crates-io \
    && cargo init \
    && cargo add empty-library \
    && rm --recursive --force /update-crates-io

RUN echo 'profile = "library"' > config.toml
# Include a built version in the docker image
RUN ./x.py doc --json --target=x86_64-unknown-linux-gnu

RUN ./x.py doc --json --target=${target}
RUN cd build/${target}/json-doc/ \
    && tar --create --verbose --file json-doc.tar *.json
