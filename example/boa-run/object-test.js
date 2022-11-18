// console.log(JSON.stringify(document, null, 2))
try {
  console.log(ToyName)
  ToyName = 'edited'
  console.log(JSON.stringify(document, null, 2))
  console.log(document.appendChild)
  console.log(document.constructor)
  console.log(document.__proto__)
  const p = document.createElement("p")
  document.children[0].appendChild(p)

  console.log(JSON.stringify(document, null, 2))
} catch(e) {
  console.log(e)
}

