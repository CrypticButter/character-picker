(ns extract-chars
  (:require
   [clojure.string :as str]))

(def file-text (slurp "extractor/DerivedName.txt"))
(def line-re #"([A-Z0-9]+)\s+;\s*([\w\s]*)")

(def output
  (apply str
         (mapcat
          (fn [line]
            (when-let [[_ cp desc] (re-matches line-re line)]
              (let [code-point (Integer/parseInt cp 16)]
                (when (< 126 code-point) ;; filter out common characters
                  (let [unic (str (.appendCodePoint (StringBuilder.) code-point))]
                   [unic " " desc \newline])))))
          (str/split-lines file-text))))

(spit "characters.txt" output)
