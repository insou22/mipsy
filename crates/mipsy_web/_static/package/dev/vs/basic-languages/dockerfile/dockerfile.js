/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/
define("vs/basic-languages/dockerfile/dockerfile", ["require"],(require)=>{
var moduleExports = (() => {
  var __defProp = Object.defineProperty;
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    __markAsModule(target);
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // src/basic-languages/dockerfile/dockerfile.ts
  var dockerfile_exports = {};
  __export(dockerfile_exports, {
    conf: () => conf,
    language: () => language
  });
  var conf = {
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"]
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" }
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" }
    ]
  };
  var language = {
    defaultToken: "",
    tokenPostfix: ".dockerfile",
    variable: /\${?[\w]+}?/,
    tokenizer: {
      root: [
        { include: "@whitespace" },
        { include: "@comment" },
        [/(ONBUILD)(\s+)/, ["keyword", ""]],
        [/(ENV)(\s+)([\w]+)/, ["keyword", "", { token: "variable", next: "@arguments" }]],
        [
          /(FROM|MAINTAINER|RUN|EXPOSE|ENV|ADD|ARG|VOLUME|LABEL|USER|WORKDIR|COPY|CMD|STOPSIGNAL|SHELL|HEALTHCHECK|ENTRYPOINT)/,
          { token: "keyword", next: "@arguments" }
        ]
      ],
      arguments: [
        { include: "@whitespace" },
        { include: "@strings" },
        [
          /(@variable)/,
          {
            cases: {
              "@eos": { token: "variable", next: "@popall" },
              "@default": "variable"
            }
          }
        ],
        [
          /\\/,
          {
            cases: {
              "@eos": "",
              "@default": ""
            }
          }
        ],
        [
          /./,
          {
            cases: {
              "@eos": { token: "", next: "@popall" },
              "@default": ""
            }
          }
        ]
      ],
      whitespace: [
        [
          /\s+/,
          {
            cases: {
              "@eos": { token: "", next: "@popall" },
              "@default": ""
            }
          }
        ]
      ],
      comment: [[/(^#.*$)/, "comment", "@popall"]],
      strings: [
        [/\\'$/, "", "@popall"],
        [/\\'/, ""],
        [/'$/, "string", "@popall"],
        [/'/, "string", "@stringBody"],
        [/"$/, "string", "@popall"],
        [/"/, "string", "@dblStringBody"]
      ],
      stringBody: [
        [
          /[^\\\$']/,
          {
            cases: {
              "@eos": { token: "string", next: "@popall" },
              "@default": "string"
            }
          }
        ],
        [/\\./, "string.escape"],
        [/'$/, "string", "@popall"],
        [/'/, "string", "@pop"],
        [/(@variable)/, "variable"],
        [/\\$/, "string"],
        [/$/, "string", "@popall"]
      ],
      dblStringBody: [
        [
          /[^\\\$"]/,
          {
            cases: {
              "@eos": { token: "string", next: "@popall" },
              "@default": "string"
            }
          }
        ],
        [/\\./, "string.escape"],
        [/"$/, "string", "@popall"],
        [/"/, "string", "@pop"],
        [/(@variable)/, "variable"],
        [/\\$/, "string"],
        [/$/, "string", "@popall"]
      ]
    }
  };
  return dockerfile_exports;
})();
return moduleExports;
});
