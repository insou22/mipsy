export function split_setup () {
  Split(["#text", "#output"], {
    direction: "vertical",
  });
  Split(["#regs", "#text_data"]);
};
