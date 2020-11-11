.PHONY : front-build front-serve build 

front-build: 
	cd front && elm make ./src/Main.elm --optimize --output=./public/elm.js
	cd front && uglifyjs ./public/elm.js --compress 'pure_funcs="F2,F3,F4,F5,F6,F7,F8,F9,A2,A3,A4,A5,A6,A7,A8,A9",pure_getters,keep_fargs=false,unsafe_comps,unsafe' | uglifyjs --mangle > ./public/elm.min.js
	mv ./front/public/elm.min.js ./front/public/elm.js

front-serve:
	cd front && elm-live src/Main.elm --hot --proxy-prefix=/ --proxy-host=http://localhost:8080 --dir=./public -- --output=public/elm.js 

back-build:
	cargo build --release --locked

back-static:
	docker run --rm -it -v $(shell pwd):/home/rust/src ekidd/rust-musl-builder cargo build --release --locked

build: front-build back-build

build-release: front-build back-static
