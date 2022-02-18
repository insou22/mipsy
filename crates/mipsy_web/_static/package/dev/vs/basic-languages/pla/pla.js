/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/
define("vs/basic-languages/pla/pla", ["require"],(require)=>{
var moduleExports = (() => {
  var __defProp = Object.defineProperty;
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    __markAsModule(target);
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // src/basic-languages/pla/pla.ts
  var pla_exports = {};
  __export(pla_exports, {
    conf: () => conf,
    language: () => language
  });
  var conf = {
    comments: {
      lineComment: "#"
    },
    brackets: [
      ["[", "]"],
      ["<", ">"],
      ["(", ")"]
    ],
    autoClosingPairs: [
      { open: "[", close: "]" },
      { open: "<", close: ">" },
      { open: "(", close: ")" }
    ],
    surroundingPairs: [
      { open: "[", close: "]" },
      { open: "<", close: ">" },
      { open: "(", close: ")" }
    ]
  };
  var language = {
    defaultToken: "",
    tokenPostfix: ".pla",
    brackets: [
      { open: "[", close: "]", token: "delimiter.square" },
      { open: "<", close: ">", token: "delimiter.angle" },
      { open: "(", close: ")", token: "delimiter.parenthesis" }
    ],
    keywords: [
      ".i",
      ".o",
      ".mv",
      ".ilb",
      ".ob",
      ".label",
      ".type",
      ".phase",
      ".pair",
      ".symbolic",
      ".symbolic-output",
      ".kiss",
      ".p",
      ".e",
      ".end"
    ],
    comment: /#.*$/,
    identifier: /[a-zA-Z]+[a-zA-Z0-9_\-]*/,
    plaContent: /[01\-~\|]+/,
    tokenizer: {
      root: [
        { include: "@whitespace" },
        [/@comment/, "comment"],
        [
          /\.([a-zA-Z_\-]+)/,
          {
            cases: {
              "@eos": { token: "keyword.$1" },
              "@keywords": {
                cases: {
                  ".type": { token: "keyword.$1", next: "@type" },
                  "@default": { token: "keyword.$1", next: "@keywordArg" }
                }
              },
              "@default": { token: "keyword.$1" }
            }
          }
        ],
        [/@identifier/, "identifier"],
        [/@plaContent/, "string"]
      ],
      whitespace: [[/[ \t\r\n]+/, ""]],
      type: [{ include: "@whitespace" }, [/\w+/, { token: "type", next: "@pop" }]],
      keywordArg: [
        [
          /[ \t\r\n]+/,
          {
            cases: {
              "@eos": { token: "", next: "@pop" },
              "@default": ""
            }
          }
        ],
        [/@comment/, "comment", "@pop"],
        [
          /[<>()\[\]]/,
          {
            cases: {
              "@eos": { token: "@brackets", next: "@pop" },
              "@default": "@brackets"
            }
          }
        ],
        [
          /\-?\d+/,
          {
            cases: {
              "@eos": { token: "number", next: "@pop" },
              "@default": "number"
            }
          }
        ],
        [
          /@identifier/,
          {
            cases: {
              "@eos": { token: "identifier", next: "@pop" },
              "@default": "identifier"
            }
          }
        ],
        [
          /[;=]/,
          {
            cases: {
              "@eos": { token: "delimiter", next: "@pop" },
              "@default": "delimiter"
            }
          }
        ]
      ]
    }
  };
  return pla_exports;
})();
return moduleExports;
});
