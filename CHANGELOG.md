# Changelog

## [0.3.0](https://github.com/nschmeller/k8s-mcp/compare/v0.2.0...v0.3.0) (2026-03-14)


### Features

* add lazy initialization for Kubernetes client with graceful error handling ([cbd6f1a](https://github.com/nschmeller/k8s-mcp/commit/cbd6f1aa4925a410c9c8d7247a6884941dcd4ce1))

## [0.2.0](https://github.com/nschmeller/k8s-mcp/compare/v0.1.0...v0.2.0) (2026-03-14)


### Features

* add support for Kubernetes v1.34 and v1.35 ([20c6ea3](https://github.com/nschmeller/k8s-mcp/commit/20c6ea35c13a6168e3744b00ee27c1e2877db6cc))

## 0.1.0 (2026-03-13)


### Features

* add CLI entry point with clap ([88723e5](https://github.com/nschmeller/k8s-mcp/commit/88723e5ffe6c914b6b7e8214e7b6bf2446986ea8))
* add core error types and library entry point ([08e164a](https://github.com/nschmeller/k8s-mcp/commit/08e164a3f42ebf04e42b402f47522810a0742397))
* add Kubernetes client and configuration module ([0d1d43d](https://github.com/nschmeller/k8s-mcp/commit/0d1d43de219723101f635a099e6c4e13a31d3109))
* add MCP protocol and server implementation ([411670a](https://github.com/nschmeller/k8s-mcp/commit/411670a941a1a1ffe2df5bc7f5016b28afe0c5ba))
* add MCP tools for Kubernetes operations ([19412b3](https://github.com/nschmeller/k8s-mcp/commit/19412b344e036316663c251eaddbf0ea48072efe))
* add output formatting module ([3f54bc1](https://github.com/nschmeller/k8s-mcp/commit/3f54bc1dd10c8dc10bc84098f0f110774d7e11df))


### Bug Fixes

* address code review findings and improve code quality ([413ad67](https://github.com/nschmeller/k8s-mcp/commit/413ad674402569ac9c2c3ecc6d832edfce6f591a))
* **ci:** remove invalid package-name input from release-please ([23ef816](https://github.com/nschmeller/k8s-mcp/commit/23ef816e96922025dfb9b99e0e8e260ae26035b6))
* **ci:** remove reference to deleted unit_tests target ([625950b](https://github.com/nschmeller/k8s-mcp/commit/625950b9b60471d3553337be2c57eeab862d845d))
* resolve clippy warnings and apply cargo fmt ([b079224](https://github.com/nschmeller/k8s-mcp/commit/b07922426f63a9f92b8ff082a485e7852cfe6b49))
* use get_optional_integer_arg for limit parameter in ListResourcesTool ([fc45dbe](https://github.com/nschmeller/k8s-mcp/commit/fc45dbe178f45ebaa605ffeba05e4cd7c5bf895b))
