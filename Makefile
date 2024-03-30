cross:
	cross build --target x86_64-pc-windows-gnu && cp target/x86_64-pc-windows-gnu/debug/jolt.exe ../../../../mnt/c/dev

search:
	curl -X POST localhost:3000/search -d '{"path": "/home/hedrickw/","pattern": ".csv"}' -H "Content-Type: application/json"

cpu:
	curl  localhost:3000/info/cpu -H "Content-Type: application/json"

memory:
	curl localhost:3000/info/memory -H "Content-Type: application/json"

large_file:
	curl -X POST localhost:3000/file/largest -d '{"path": "/mnt/c/ProgramData/Application Data"}' -H "Content-Type: application/json"

