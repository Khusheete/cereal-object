# About

Cereal Object (CE) is a serialization format inspired by JSON that I made for the godot engine.

# Why make another serialization format ?

While the JSON format works well in most cases, it may be a bit clunky to use when encoding some data types (colors, vectors...) or when the object types are important. CE was made to support godot types while feeling and looking like JSON.

# How do I use it ?


This plugin was made as a gdextension and is programmed in rust. To use it, you will need the dynamic library produced by the rust code.

## Compiling from source

You can compile the rust code into a dynamic library by using the makefile located in the `rust` directory. It will use your rust install to compile it, which means that you will need to have rustc installed alongside cargo.

## Getting the source

If you do not want or cannot compile the rust code, the binaries for the latest version *should* be available as a github release. The binaries should be placed in a folder named `bin` inside this directory (you may need to create it).

## A Simple API

Once this is done, you should be able to use `CerealObject.stringify` to serialize a variant, and `CerealObject.parse_string` to parse a CE string:

```gdscript
var datastruct: Dictionary = {
	"a": "b",
	"boolean": false,
	"not_boolean": true,
	"super duper complex name": true,
	"string_array": PackedStringArray(["a", "b", "c"]),
	"float_array": PackedFloat64Array([1., 2., 3., -6e-7]),
	"int_array": PackedInt32Array([2, 1, 3, 4, 7, 11, 18, 29, 47]),
	"array": [
		"A", 2, 3.0, {
			"nested_stuff": 3.141592654,
			"super_mega_nested": {
				"secret": &"password"
			}
		}
	],
	"rect": Rect2(1, 2, 3.1415, 4),
	"recti": Rect2i(666, -42, 47, 32),
	"some vector": Vector2i(6, 7),
	"aabb": AABB(Vector3(-5.2, 7.9, PI), Vector3(12.5, 1, -95.2)),
	"transform": Transform3D(
			Vector3(0.09597263, 0.5049345, 0.132819),
			Vector3(0.1372523, 0.8690669, 0.55455375),
			Vector3(0.96561974, 0.7665665, 0.1779638),
			Vector3(0.64600086, 0.8919131, 0.3106943),
	),
	"crimson": Color.CRIMSON,
}

var string: String = CerealObject.stringify(datastruct)
```

```gdscript
var ce_string: String = """{
	a: "b",
	boolean: false,
	not_boolean: true,
	"super duper complex name": true,
	string_array: [String,
		"a",
		"b",
		"c"
	],
	float_array: [f64,
		1f,
		2f,
		3f,
		-0.0000006f
	],
	int_array: [i32,
		2,
		1,
		3,
		4,
		7,
		11,
		18,
		29,
		47
	],
	array: [
		"A",
		2,
		3f,
		{
			nested_stuff: 3.141592654f,
			super_mega_nested: {
				secret: "password"
			}
		}
	],
	rect: Rect2(1f, 2f; 3.1415f, 4f),
	recti: Rect2i(666i, -42i; 47i, 32i),
	"some vector": Vector2i(6i, 7i),
	aabb: AABB(-5.2f, 7.9f, 3.1415927f; 12.5f, 1f, -95.2f),
	transform: Transform3D(0.09597263f, 0.5049345f, 0.132819f; 0.1372523f, 0.8690669f, 0.55455375f; 0.96561974f, 0.7665665f, 0.1779638f; 0.64600086f, 0.8919131f, 0.3106943f),
	crimson: Color(220, 20, 60, 255)
}"""

var parsed: Dictionary = CerealObject.parse_string(ce_string)
```