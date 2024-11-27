var
  ws = require('ws'),
  buf = [],
  client = new ws("ws://localhost:3000")

process.stdin
.on('readable', stdioRead)
.on('end', Close)
.on('error', Close)

client
.on('open', wsOpen)
.on('message', wsMsg)
.on('close', Close)
.on('error', Close)

function stdioRead()
{
  var x
  while(null!=(x=this.read()))
    if(buf)
      buf.push(x)
    else
      client.send(x)
}

function wsOpen()
{
  buf.forEach(function(data){ client.send(data) })
  buf = null
}

function wsMsg(data)
{
  console.log(data);
  process.stdout.write(data)
}

function Close()
{
  process.exit()
}