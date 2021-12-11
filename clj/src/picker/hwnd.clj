(ns picker.hwnd
  (:require
   [coffi.mem :as mem :refer [defalias]]
   [coffi.ffi :as ffi :refer [defcfn]]))

(ffi/load-library "../target/debug/libpicker.dll")

(defcfn make-hwnd
  picker_make_hwnd [] ::mem/int)
