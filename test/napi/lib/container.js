const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

describe("Container type extractors", function () {
  it("can produce and consume a RefCell", function () {
    const cell = addon.createStringRefCell("my sekret mesij");
    const s = addon.readStringRefCell(cell);
    assert.strictEqual(s, "my sekret mesij");
  });

  it("can produce and modify a RefCell", function () {
    const cell = addon.createStringRefCell("new");
    addon.writeStringRefCell(cell, "modified");
    assert.strictEqual(addon.readStringRefCell(cell), "modified");
  });

  it("can concatenate a RefCell<String> with a String", function () {
    const cell = addon.createStringRefCell("hello");
    const s = addon.stringRefCellConcat(cell, " world");
    assert.strictEqual(s, "hello world");
  });

  it("fail with a type error when not given a RefCell", function () {
    try {
      addon.stringRefCellConcat("hello", " world");
      assert.fail("should have thrown");
    } catch (err) {
      assert.instanceOf(err, TypeError);
      assert.strictEqual(
        err.message,
        "expected neon::types_impl::boxed::JsBox<core::cell::RefCell<alloc::string::String>>"
      );
    }
  });

  it("dynamically fail when borrowing a mutably borrowed RefCell", function () {
    const cell = addon.createStringRefCell("hello");
    try {
      addon.borrowMutAndThen(cell, () => {
        addon.stringRefConcat(cell, " world");
      });
      assert.fail("should have thrown");
    } catch (err) {
      assert.instanceOf(err, Error);
      assert.include(err.message, "already mutably borrowed");
    }
  });

  it("dynamically fail when modifying a borrowed RefCell", function () {
    const cell = addon.createStringRefCell("hello");
    try {
      addon.borrowAndThen(cell, () => {
        addon.writeStringRef(cell, "world");
      });
      assert.fail("should have thrown");
    } catch (err) {
      assert.instanceOf(err, Error);
      assert.include(err.message, "already borrowed");
    }
  });

  it("can produce and consume an Rc", function () {
    const cell = addon.createStringRc("my sekret mesij");
    const s = addon.readStringRc(cell);
    assert.strictEqual(s, "my sekret mesij");
  });

  it("can produce and consume an Arc", function () {
    const cell = addon.createStringArc("my sekret mesij");
    const s = addon.readStringArc(cell);
    assert.strictEqual(s, "my sekret mesij");
  });
});
