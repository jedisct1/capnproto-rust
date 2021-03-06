/*
 * Copyright (c) 2013-2014, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

use std;
use common::*;
use endian::*;
use message::*;
use arena;
use io;


pub struct OwnedSpaceMessageReader {
    priv options : ReaderOptions,
    priv arena : ~arena::ReaderArena,
    priv segment_slices : ~[(uint, uint)],
    priv owned_space : ~[Word],
}

impl MessageReader for OwnedSpaceMessageReader {
    fn get_segment<'b>(&'b self, id : uint) -> &'b [Word] {
        let (a,b) = self.segment_slices[id];
        self.owned_space.slice(a, b)
    }

    fn arena<'b>(&'b self) -> &'b arena::ReaderArena { &*self.arena }
    fn mut_arena<'b>(&'b mut self) -> &'b mut arena::ReaderArena { &mut *self.arena }

    fn get_options<'b>(&'b self) -> &'b ReaderOptions {
        return &self.options;
    }
}

fn invalid_input<T>(desc : &'static str) -> std::io::IoResult<T> {
    return Err(std::io::IoError{ kind : std::io::InvalidInput,
                                 desc : desc,
                                 detail : None});
}

pub fn new_reader<U : std::io::Reader>(inputStream : &mut U,
                                       options : ReaderOptions)
                                       -> std::io::IoResult<OwnedSpaceMessageReader> {

    let firstWord = if_ok!(inputStream.read_bytes(8));

    let segmentCount : u32 =
        unsafe {let p : *WireValue<u32> = std::cast::transmute(firstWord.as_ptr());
                (*p).get() + 1
    };

    let segment0Size =
        if segmentCount == 0 { 0 } else {
        unsafe {let p : *WireValue<u32> = std::cast::transmute(firstWord.unsafe_ref(4));
                (*p).get()
        }
    };

    let mut totalWords = segment0Size;

    if segmentCount >= 512 {
        return invalid_input("too many segments");
    }

    let mut moreSizes : ~[u32] = std::vec::from_elem((segmentCount & !1) as uint, 0u32);

    if segmentCount > 1 {
        let moreSizesRaw = if_ok!(inputStream.read_bytes((4 * (segmentCount & !1)) as uint));
        for ii in range(0, segmentCount as uint - 1) {
            moreSizes[ii] = unsafe {
                let p : *WireValue<u32> =
                    std::cast::transmute(moreSizesRaw.unsafe_ref(ii * 4));
                (*p).get()
            };
            totalWords += moreSizes[ii];
        }
    }

    //# Don't accept a message which the receiver couldn't possibly
    //# traverse without hitting the traversal limit. Without this
    //# check, a malicious client could transmit a very large
    //# segment size to make the receiver allocate excessive space
    //# and possibly crash.
    if ! (totalWords as u64 <= options.traversalLimitInWords)  {
        return invalid_input("Message is too large. To increase the limit on the \
                              receiving end, see capnp::ReaderOptions.");
    }

    let mut ownedSpace : ~[Word] = allocate_zeroed_words(totalWords as uint);
    let bufLen = totalWords as uint * BYTES_PER_WORD;

    unsafe {
        let ptr : *mut u8 = std::cast::transmute(ownedSpace.unsafe_mut_ref(0));
        if_ok!(std::vec::raw::mut_buf_as_slice::<u8,std::io::IoResult<uint>>(ptr, bufLen, |buf| {
                    io::read_at_least(inputStream, buf, bufLen)
                }));
    }

    // TODO(maybe someday) lazy reading like in capnp-c++?

    let mut segment_slices : ~[(uint, uint)] = ~[(0, segment0Size as uint)];

    let arena = {
        let segment0 : &[Word] = ownedSpace.slice(0, segment0Size as uint);
        let mut segments : ~[&[Word]] = ~[segment0];

        if segmentCount > 1 {
            let mut offset = segment0Size;

            for ii in range(0, segmentCount as uint - 1) {
                segments.push(ownedSpace.slice(offset as uint,
                                               (offset + moreSizes[ii]) as uint));
                segment_slices.push((offset as uint,
                                     (offset + moreSizes[ii]) as uint));
                offset += moreSizes[ii];
            }
        }
        arena::ReaderArena::new(segments)
    };

    Ok(OwnedSpaceMessageReader {
        segment_slices : segment_slices,
        owned_space : ownedSpace,
        arena : arena,
        options : options,
    })
}


pub fn write_message<T : std::io::Writer, U : MessageBuilder>(
    outputStream : &mut T,
    message : &U) {

    message.get_segments_for_output(
        |segments| {

            let tableSize : uint = ((segments.len() + 2) & (!1));

            let mut table : ~[WireValue<u32>] = std::vec::with_capacity(tableSize);
            unsafe { table.set_len(tableSize) }

            table[0].set((segments.len() - 1) as u32);

            for i in range(0, segments.len()) {
                table[i + 1].set(segments[i].len() as u32);
            }
            if segments.len() % 2 == 0 {
                // Set padding.
                table[segments.len() + 1].set( 0 );
            }

            unsafe {
                let ptr : *u8 = std::cast::transmute(table.unsafe_ref(0));
                std::vec::raw::buf_as_slice::<u8,()>(ptr, table.len() * 4, |buf| {
                        outputStream.write(buf).unwrap();
                    })
            }

            for i in range(0, segments.len()) {
                unsafe {
                    let ptr : *u8 = std::cast::transmute(segments[i].unsafe_ref(0));
                    std::vec::raw::buf_as_slice::<u8,()>(
                        ptr,
                        segments[i].len() * BYTES_PER_WORD,
                        |buf| { outputStream.write(buf).unwrap(); });
                }
            }
        });
}
