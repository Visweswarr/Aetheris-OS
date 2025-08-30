load("@aspect_rules_ts//ts:defs.bzl", "ts_project")
load("@aspect_rules_js//js:defs.bzl", "js_binary")

ts_project(
    name = "as_parse_ts",
    srcs = ["index.ts"],
    declaration = True,
    tsconfig = "//:tsconfig.json",
    deps = [],
)

js_binary(
    name = "as-parse",
    data = [":as_parse_ts"],
    entry_point = "index.ts",
    visibility = ["//visibility:public"],
)
