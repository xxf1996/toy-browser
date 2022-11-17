// console.log(JSON.stringify(document, null, 2))
console.log(ToyName)
ToyName = 'edited'
document.children[0].appendChild({
  type: 'p',
  children: []
})

console.log(JSON.stringify(document, null, 2))

