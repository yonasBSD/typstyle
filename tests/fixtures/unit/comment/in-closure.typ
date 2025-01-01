#let conf(
  title: none, //comments
  authors: (),
  abstract: [],
  lang: "zh",   // language
  doctype: "book",  //comments
  doc  // all comments will be kept by typstyle
)={doc}

#let f()/* 0 */=/* 1 */   ()=>  /* 2 */none
#let g(..)/* 0 */   =  /* 1 */  ()=>  /* 2 */none
#let h(..)/* 0 */ =/* 1 */ ()=>/* 2 */   { none}

#let f = /* 0 */ ()/* 1 */=>  /* 2 */none
#let g =/* 0 */(..)/* 1 */=>/* 2 */   none
#let h =/* 0 */(..)/* 1 */=>  /* 2 */   { none}