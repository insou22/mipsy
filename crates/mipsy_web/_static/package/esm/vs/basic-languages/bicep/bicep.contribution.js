/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/

// src/basic-languages/bicep/bicep.contribution.ts
import { registerLanguage } from "../_.contribution.js";
registerLanguage({
  id: "bicep",
  extensions: [".bicep"],
  aliases: ["Bicep"],
  loader: () => {
    if (false) {
      return new Promise((resolve, reject) => {
        __require(["vs/basic-languages/bicep/bicep"], resolve, reject);
      });
    } else {
      return import("./bicep.js");
    }
  }
});
