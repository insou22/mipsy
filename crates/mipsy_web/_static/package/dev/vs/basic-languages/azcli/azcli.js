/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/
define("vs/basic-languages/azcli/azcli", ["require"],(require)=>{
var moduleExports = (() => {
  var __defProp = Object.defineProperty;
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    __markAsModule(target);
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // src/basic-languages/azcli/azcli.ts
  var azcli_exports = {};
  __export(azcli_exports, {
    conf: () => conf,
    language: () => language
  });
  var conf = {
    comments: {
      lineComment: "#"
    }
  };
  var language = {
    defaultToken: "keyword",
    ignoreCase: true,
    tokenPostfix: ".azcli",
    str: /[^#\s]/,
    tokenizer: {
      root: [
        { include: "@comment" },
        [
          /\s-+@str*\s*/,
          {
            cases: {
              "@eos": { token: "key.identifier", next: "@popall" },
              "@default": { token: "key.identifier", next: "@type" }
            }
          }
        ],
        [
          /^-+@str*\s*/,
          {
            cases: {
              "@eos": { token: "key.identifier", next: "@popall" },
              "@default": { token: "key.identifier", next: "@type" }
            }
          }
        ]
      ],
      type: [
        { include: "@comment" },
        [
          /-+@str*\s*/,
          {
            cases: {
              "@eos": { token: "key.identifier", next: "@popall" },
              "@default": "key.identifier"
            }
          }
        ],
        [
          /@str+\s*/,
          {
            cases: {
              "@eos": { token: "string", next: "@popall" },
              "@default": "string"
            }
          }
        ]
      ],
      comment: [
        [
          /#.*$/,
          {
            cases: {
              "@eos": { token: "comment", next: "@popall" }
            }
          }
        ]
      ]
    }
  };
  return azcli_exports;
})();
return moduleExports;
});
