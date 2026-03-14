# Changelog

## [0.5.0](https://github.com/nschmeller/k8s-mcp/compare/v0.4.0...v0.5.0) (2026-03-14)


### Features

* add CLI entry point with clap ([88723e5](https://github.com/nschmeller/k8s-mcp/commit/88723e5ffe6c914b6b7e8214e7b6bf2446986ea8))
* add core error types and library entry point ([08e164a](https://github.com/nschmeller/k8s-mcp/commit/08e164a3f42ebf04e42b402f47522810a0742397))
* add Kubernetes client and configuration module ([0d1d43d](https://github.com/nschmeller/k8s-mcp/commit/0d1d43de219723101f635a099e6c4e13a31d3109))
* add lazy initialization for Kubernetes client with graceful error handling ([cbd6f1a](https://github.com/nschmeller/k8s-mcp/commit/cbd6f1aa4925a410c9c8d7247a6884941dcd4ce1))
* add MCP protocol and server implementation ([411670a](https://github.com/nschmeller/k8s-mcp/commit/411670a941a1a1ffe2df5bc7f5016b28afe0c5ba))
* add MCP tools for Kubernetes operations ([19412b3](https://github.com/nschmeller/k8s-mcp/commit/19412b344e036316663c251eaddbf0ea48072efe))
* add output formatting module ([3f54bc1](https://github.com/nschmeller/k8s-mcp/commit/3f54bc1dd10c8dc10bc84098f0f110774d7e11df))
* add support for Kubernetes v1.34 and v1.35 ([20c6ea3](https://github.com/nschmeller/k8s-mcp/commit/20c6ea35c13a6168e3744b00ee27c1e2877db6cc))
* set Kubernetes v1.35 as default version ([3299baf](https://github.com/nschmeller/k8s-mcp/commit/3299baf81d5e9d2e7740ccaf99f8fa8f6364ad7d))


### Bug Fixes

* `release-please` do not include name in tag ([99925c4](https://github.com/nschmeller/k8s-mcp/commit/99925c409c13f4230f90e1f05028423b04b0aba4))
* `release-please` draft releases ([d10ca1b](https://github.com/nschmeller/k8s-mcp/commit/d10ca1b3a133edea6455386c19207cb7c7d266a7))
* add `release-please` manifest ([c27917f](https://github.com/nschmeller/k8s-mcp/commit/c27917fe45231613a3b44a1d189bc8b33fc29aa2))
* address code review findings and improve code quality ([413ad67](https://github.com/nschmeller/k8s-mcp/commit/413ad674402569ac9c2c3ecc6d832edfce6f591a))
* **ci:** remove invalid package-name input from release-please ([23ef816](https://github.com/nschmeller/k8s-mcp/commit/23ef816e96922025dfb9b99e0e8e260ae26035b6))
* **ci:** remove reference to deleted unit_tests target ([625950b](https://github.com/nschmeller/k8s-mcp/commit/625950b9b60471d3553337be2c57eeab862d845d))
* correctly upload artifacts to immutable releases ([86b9b76](https://github.com/nschmeller/k8s-mcp/commit/86b9b765138a5bff2b0081da843ffbf8bf3df2e7))
* decouple releasing from `release-please` ([afb4820](https://github.com/nschmeller/k8s-mcp/commit/afb482051a313b2ceb6125ae85411c8f412841c5))
* no immutable releases (for now) ([f72eb45](https://github.com/nschmeller/k8s-mcp/commit/f72eb45f4419badac6ec606e38c20d185cb6adfa))
* reset releases state ([8620bdc](https://github.com/nschmeller/k8s-mcp/commit/8620bdca0c2ae8bf241f54e357a029821e7b4078))
* resolve clippy warnings and apply cargo fmt ([b079224](https://github.com/nschmeller/k8s-mcp/commit/b07922426f63a9f92b8ff082a485e7852cfe6b49))
* update release manifests ([2d4a1fd](https://github.com/nschmeller/k8s-mcp/commit/2d4a1fd0f5da0273c2496d5200509fad1a413437))
* use get_optional_integer_arg for limit parameter in ListResourcesTool ([fc45dbe](https://github.com/nschmeller/k8s-mcp/commit/fc45dbe178f45ebaa605ffeba05e4cd7c5bf895b))

## [0.7.0](https://github.com/nschmeller/k8s-mcp/compare/v0.6.0...v0.7.0) (2026-03-14)


### Features

* add CLI entry point with clap ([88723e5](https://github.com/nschmeller/k8s-mcp/commit/88723e5ffe6c914b6b7e8214e7b6bf2446986ea8))
* add core error types and library entry point ([08e164a](https://github.com/nschmeller/k8s-mcp/commit/08e164a3f42ebf04e42b402f47522810a0742397))
* add Kubernetes client and configuration module ([0d1d43d](https://github.com/nschmeller/k8s-mcp/commit/0d1d43de219723101f635a099e6c4e13a31d3109))
* add lazy initialization for Kubernetes client with graceful error handling ([cbd6f1a](https://github.com/nschmeller/k8s-mcp/commit/cbd6f1aa4925a410c9c8d7247a6884941dcd4ce1))
* add MCP protocol and server implementation ([411670a](https://github.com/nschmeller/k8s-mcp/commit/411670a941a1a1ffe2df5bc7f5016b28afe0c5ba))
* add MCP tools for Kubernetes operations ([19412b3](https://github.com/nschmeller/k8s-mcp/commit/19412b344e036316663c251eaddbf0ea48072efe))
* add output formatting module ([3f54bc1](https://github.com/nschmeller/k8s-mcp/commit/3f54bc1dd10c8dc10bc84098f0f110774d7e11df))
* add support for Kubernetes v1.34 and v1.35 ([20c6ea3](https://github.com/nschmeller/k8s-mcp/commit/20c6ea35c13a6168e3744b00ee27c1e2877db6cc))
* set Kubernetes v1.35 as default version ([3299baf](https://github.com/nschmeller/k8s-mcp/commit/3299baf81d5e9d2e7740ccaf99f8fa8f6364ad7d))


### Bug Fixes

* `release-please` do not include name in tag ([99925c4](https://github.com/nschmeller/k8s-mcp/commit/99925c409c13f4230f90e1f05028423b04b0aba4))
* add `release-please` manifest ([c27917f](https://github.com/nschmeller/k8s-mcp/commit/c27917fe45231613a3b44a1d189bc8b33fc29aa2))
* address code review findings and improve code quality ([413ad67](https://github.com/nschmeller/k8s-mcp/commit/413ad674402569ac9c2c3ecc6d832edfce6f591a))
* **ci:** remove invalid package-name input from release-please ([23ef816](https://github.com/nschmeller/k8s-mcp/commit/23ef816e96922025dfb9b99e0e8e260ae26035b6))
* **ci:** remove reference to deleted unit_tests target ([625950b](https://github.com/nschmeller/k8s-mcp/commit/625950b9b60471d3553337be2c57eeab862d845d))
* correctly upload artifacts to immutable releases ([86b9b76](https://github.com/nschmeller/k8s-mcp/commit/86b9b765138a5bff2b0081da843ffbf8bf3df2e7))
* decouple releasing from `release-please` ([afb4820](https://github.com/nschmeller/k8s-mcp/commit/afb482051a313b2ceb6125ae85411c8f412841c5))
* resolve clippy warnings and apply cargo fmt ([b079224](https://github.com/nschmeller/k8s-mcp/commit/b07922426f63a9f92b8ff082a485e7852cfe6b49))
* use get_optional_integer_arg for limit parameter in ListResourcesTool ([fc45dbe](https://github.com/nschmeller/k8s-mcp/commit/fc45dbe178f45ebaa605ffeba05e4cd7c5bf895b))

## [0.6.0](https://github.com/nschmeller/k8s-mcp/compare/v0.5.0...v0.6.0) (2026-03-14)


### Features

* add CLI entry point with clap ([88723e5](https://github.com/nschmeller/k8s-mcp/commit/88723e5ffe6c914b6b7e8214e7b6bf2446986ea8))
* add core error types and library entry point ([08e164a](https://github.com/nschmeller/k8s-mcp/commit/08e164a3f42ebf04e42b402f47522810a0742397))
* add Kubernetes client and configuration module ([0d1d43d](https://github.com/nschmeller/k8s-mcp/commit/0d1d43de219723101f635a099e6c4e13a31d3109))
* add lazy initialization for Kubernetes client with graceful error handling ([cbd6f1a](https://github.com/nschmeller/k8s-mcp/commit/cbd6f1aa4925a410c9c8d7247a6884941dcd4ce1))
* add MCP protocol and server implementation ([411670a](https://github.com/nschmeller/k8s-mcp/commit/411670a941a1a1ffe2df5bc7f5016b28afe0c5ba))
* add MCP tools for Kubernetes operations ([19412b3](https://github.com/nschmeller/k8s-mcp/commit/19412b344e036316663c251eaddbf0ea48072efe))
* add output formatting module ([3f54bc1](https://github.com/nschmeller/k8s-mcp/commit/3f54bc1dd10c8dc10bc84098f0f110774d7e11df))
* add support for Kubernetes v1.34 and v1.35 ([20c6ea3](https://github.com/nschmeller/k8s-mcp/commit/20c6ea35c13a6168e3744b00ee27c1e2877db6cc))
* set Kubernetes v1.35 as default version ([3299baf](https://github.com/nschmeller/k8s-mcp/commit/3299baf81d5e9d2e7740ccaf99f8fa8f6364ad7d))


### Bug Fixes

* `release-please` do not include name in tag ([99925c4](https://github.com/nschmeller/k8s-mcp/commit/99925c409c13f4230f90e1f05028423b04b0aba4))
* add `release-please` manifest ([c27917f](https://github.com/nschmeller/k8s-mcp/commit/c27917fe45231613a3b44a1d189bc8b33fc29aa2))
* address code review findings and improve code quality ([413ad67](https://github.com/nschmeller/k8s-mcp/commit/413ad674402569ac9c2c3ecc6d832edfce6f591a))
* **ci:** remove invalid package-name input from release-please ([23ef816](https://github.com/nschmeller/k8s-mcp/commit/23ef816e96922025dfb9b99e0e8e260ae26035b6))
* **ci:** remove reference to deleted unit_tests target ([625950b](https://github.com/nschmeller/k8s-mcp/commit/625950b9b60471d3553337be2c57eeab862d845d))
* correctly upload artifacts to immutable releases ([86b9b76](https://github.com/nschmeller/k8s-mcp/commit/86b9b765138a5bff2b0081da843ffbf8bf3df2e7))
* resolve clippy warnings and apply cargo fmt ([b079224](https://github.com/nschmeller/k8s-mcp/commit/b07922426f63a9f92b8ff082a485e7852cfe6b49))
* use get_optional_integer_arg for limit parameter in ListResourcesTool ([fc45dbe](https://github.com/nschmeller/k8s-mcp/commit/fc45dbe178f45ebaa605ffeba05e4cd7c5bf895b))

## [0.5.0](https://github.com/nschmeller/k8s-mcp/compare/k8s-mcp-v0.4.0...k8s-mcp-v0.5.0) (2026-03-14)


### Features

* add CLI entry point with clap ([88723e5](https://github.com/nschmeller/k8s-mcp/commit/88723e5ffe6c914b6b7e8214e7b6bf2446986ea8))
* add core error types and library entry point ([08e164a](https://github.com/nschmeller/k8s-mcp/commit/08e164a3f42ebf04e42b402f47522810a0742397))
* add Kubernetes client and configuration module ([0d1d43d](https://github.com/nschmeller/k8s-mcp/commit/0d1d43de219723101f635a099e6c4e13a31d3109))
* add lazy initialization for Kubernetes client with graceful error handling ([cbd6f1a](https://github.com/nschmeller/k8s-mcp/commit/cbd6f1aa4925a410c9c8d7247a6884941dcd4ce1))
* add MCP protocol and server implementation ([411670a](https://github.com/nschmeller/k8s-mcp/commit/411670a941a1a1ffe2df5bc7f5016b28afe0c5ba))
* add MCP tools for Kubernetes operations ([19412b3](https://github.com/nschmeller/k8s-mcp/commit/19412b344e036316663c251eaddbf0ea48072efe))
* add output formatting module ([3f54bc1](https://github.com/nschmeller/k8s-mcp/commit/3f54bc1dd10c8dc10bc84098f0f110774d7e11df))
* add support for Kubernetes v1.34 and v1.35 ([20c6ea3](https://github.com/nschmeller/k8s-mcp/commit/20c6ea35c13a6168e3744b00ee27c1e2877db6cc))
* set Kubernetes v1.35 as default version ([3299baf](https://github.com/nschmeller/k8s-mcp/commit/3299baf81d5e9d2e7740ccaf99f8fa8f6364ad7d))


### Bug Fixes

* add `release-please` manifest ([c27917f](https://github.com/nschmeller/k8s-mcp/commit/c27917fe45231613a3b44a1d189bc8b33fc29aa2))
* address code review findings and improve code quality ([413ad67](https://github.com/nschmeller/k8s-mcp/commit/413ad674402569ac9c2c3ecc6d832edfce6f591a))
* **ci:** remove invalid package-name input from release-please ([23ef816](https://github.com/nschmeller/k8s-mcp/commit/23ef816e96922025dfb9b99e0e8e260ae26035b6))
* **ci:** remove reference to deleted unit_tests target ([625950b](https://github.com/nschmeller/k8s-mcp/commit/625950b9b60471d3553337be2c57eeab862d845d))
* correctly upload artifacts to immutable releases ([86b9b76](https://github.com/nschmeller/k8s-mcp/commit/86b9b765138a5bff2b0081da843ffbf8bf3df2e7))
* resolve clippy warnings and apply cargo fmt ([b079224](https://github.com/nschmeller/k8s-mcp/commit/b07922426f63a9f92b8ff082a485e7852cfe6b49))
* use get_optional_integer_arg for limit parameter in ListResourcesTool ([fc45dbe](https://github.com/nschmeller/k8s-mcp/commit/fc45dbe178f45ebaa605ffeba05e4cd7c5bf895b))

## [0.4.0](https://github.com/nschmeller/k8s-mcp/compare/v0.3.0...v0.4.0) (2026-03-14)


### Features

* set Kubernetes v1.35 as default version ([3299baf](https://github.com/nschmeller/k8s-mcp/commit/3299baf81d5e9d2e7740ccaf99f8fa8f6364ad7d))

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
