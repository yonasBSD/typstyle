#let a = (b: (f: (body, theme: auto) => none))
#let c = a.b

#show: a.b.f.with(theme: {
  let t = none
  t
})

#show: c.f.with(theme: {
  let t = none
  t
})

#show: a/* 1 */./* 2 */b/* 3 */./* 4 */f/* 5 */.with(theme: {
  let t = none
  t
})

#show: c/* 1 */./* 2 */f/* 3 */.with(theme: {
  let t = none
  t
})

#{
show: a.b.f.with(theme: {
  let t = none
  t
})

show: c.f.with(theme: {
  let t = none
  t
})

show: a/* 1 */./* 2 */b/* 3 */./* 4 */f/* 5 */.with(theme: {
  let t = none
  t
})

show: c/* 1 */./* 2 */f/* 3 */.with(theme: {
  let t = none
  t
})
}
