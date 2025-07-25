# Encoding

[https://github.com/pingcap/talent-plan/blob/master/courses/rust/building-blocks/bb-2.md](https://github.com/pingcap/talent-plan/blob/master/courses/rust/building-blocks/bb-2.md)

## Exercise: Serialize and deserialize a data structure with serde (JSON).

This exercise and the next two will introduce basic serialization and deserialization with serde. serde serializes data quickly and is easy to use, while also being extensible and expressive.

For your serializable data structure, imagine a flat game-playing surface covered in a grid of squares, like a chess board. Imagine you have a game character that every turn may move any number of squares in a single direction. Define a type, Move that represents a single move of that character.

Derive the Debug trait so Move is easily printable with the {:?} format specifier.

Write a main function that defines a variable, a, of type Move, serializes it with serde to a File, then deserializes it back again to a variable, b, also of type Move.

Use JSON as the serialization format.

Print a and b with println! and the {:?} format specifier to verify successful deserialization.

Note that the serde book has many examples to work off of.

## Exercise: Serialize and deserialize a data structure to a buffer with serde (RON).

Do the same as above, except this time, instead of serializing to a File, serialize to a Vec<u8> buffer, and after that try using RON instead of JSON as the format. Are there any differences in serialization to a Vec instead of a File? What about in using the RON crate vs the JSON crate?

Convert the Vec<u8> to String with str::from_utf8, unwrapping the result, then print that serialized string representation to see what Move looks like serialized to RON.

## Exercise: Serialize and deserialize 1000 data structures with serde (BSON).

This one is slightly different. Where the previous exercises serialized and deserialized a single value to a buffer, in this one serialize 1000 different Move values to a single file, back-to-back, then deserialize them again. This time use the BSON format.

Things to discover here are whether serde automatically maintains the correct file offsets (the "cursor") to deserialize multiple values in sequence, or if you need to parse your own "frames" around each value to define their size, and how to detect that there are no more values to parse at the end of the file.

After you've succeeded at serializing and deserializing multiple values to a file, try it again with a Vec<u8>. Serializing and deserializing generally requires the destination implement the Write and Read traits. Does Vec<u8> implement either or both? What is the behavior of those implementations? You may need to wrap your buffer in wrapper types that implement these traits in order to get the correct behavior — the API docs for the traits list all their implementors in the standard library, and whatever you need will be in there somewhere.

