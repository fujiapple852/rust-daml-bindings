NAMESPACE = fujiapple
REPOSITORY = rust-musl
TAG = 1.53-v1
.PHONY: build push
build-image:
	docker build --network host -t ${NAMESPACE}/${REPOSITORY}:${TAG} -t ${NAMESPACE}/${REPOSITORY}:latest .
push:
	docker push ${NAMESPACE}/${REPOSITORY}:${TAG}
build:
	docker run -it -v $(shell pwd)../../../:/rust --name daml-oas-build --rm fujiapple/rust-musl:latest
run-oas:
	docker run -it --rm -v $(shell pwd)../../../:/rust centos /rust/examples/daml-oas/target/x86_64-unknown-linux-musl/release/daml-oas $(dar)