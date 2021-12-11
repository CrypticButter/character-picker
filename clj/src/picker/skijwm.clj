(ns picker.skijwm
  (:import
   (io.github.humbleui.jwm App Event EventWindowCloseRequest EventWindowScreenChange EventWindowResize EventFrame LayerD3D12 ZOrder)
   (io.github.humbleui.skija BackendRenderTarget ColorSpace DirectContext FramebufferFormat PixelGeometry Surface SurfaceColorFormat SurfaceOrigin SurfaceProps)
   (java.util.function Consumer)))

(defprotocol SkiaLayer
  (slayer [layer])
  (attach [layer window])
  (before-paint [layer])
  (after-paint [layer])
  (resize [layer width height])
  (close [layer]))

(deftype SkiaLayerD3D12 [^:unsynchronized-mutable directContext
                         ^:unsynchronized-mutable renderTarget
                         ^:unsynchronized-mutable surface
                         ^LayerD3D12 layer]
  SkiaLayer
  (slayer [_] layer)
  (attach [_ window]
    (.attach layer window)
    (set! directContext (DirectContext/makeDirect3D (.getAdapterPtr layer), (.getDevicePtr layer), (.getQueuePtr layer))))
  (before-paint [_]
    (set! renderTarget (BackendRenderTarget/makeDirect3D
                        (.getWidth layer)
                        (.getHeight layer)
                        (.nextDrawableTexturePtr layer)
                        (.getFormat layer)
                        (.getSampleCount layer)
                        (.getLevelCount layer)))

    (set! surface (Surface/makeFromBackendRenderTarget
                   directContext
                   renderTarget
                   SurfaceOrigin/TOP_LEFT
                   SurfaceColorFormat/RGBA_8888
                   (ColorSpace/getSRGB)
                   (SurfaceProps. PixelGeometry/RGB_H)))

    (.getCanvas surface))
  (after-paint [_]
    (.flush surface)
    (.submit directContext false)
    (.swapBuffers layer)

    (.close surface)
    (.close renderTarget))
  (resize [_ width height]
    (.submit directContext true)
    (.resize layer width height))
  (close [_]
    (when directContext
      (.abandon directContext)
      (.close directContext))
    (.close layer)))

(defn draw [& {:keys [canvas width height scale mousex mousey]}]
  (.clear canvas 0xFFFFFFFF)
  (let [layer_int (.save canvas)
          ;; scene (currentScene)
        ]
      #_(when (.scale scene)
        (.scale canvas scale scale))
      #_(.draw scene canvas width height scale mousex mousey)
      (.restoreToCount canvas layer_int)
      #_(.tick hud)
      #_(if stats
        (let [layer_int (.save canvas)]
          (.scale canvas scale scale)
          (.draw hud canbas scene width height)
          (.restoreToCount layer))
        (.log hud))))

(defprotocol MainP
  (set-width [_ v])
  (set-height [_ v])
  (get-width [_])
  (get-layer [_])
  (set-layer [_ v])
  (set-scale [_ v])
  (get-scale [_])
  (get-height [_]))

(deftype Main [^:unsynchronized-mutable width
               ^:unsynchronized-mutable height
               ^:unsynchronized-mutable scale
               xpos
               ypos
               ^:unsynchronized-mutable layer]
  MainP
  (set-width [_ v] (set! width v))
  (set-height [_ v] (set! height v))
  (set-scale [_ v] (set! scale v))
  (get-scale [_] scale)
  (get-layer [_] layer)
  (set-layer [_ v] (set! layer v))
  (get-width [_] width)
  (get-height [_] height))

(defn paint [layer ^Main mainobj]
  (when layer
    (let [canvas (before-paint layer)
          width (get-width mainobj)
          height (get-height mainobj)
          scale (get-scale mainobj)
          xpos (.-xpos mainobj)
          ypos (.-ypos mainobj)]
      (draw :canvas canvas :width width
            :height height :scale scale
            :mousex (max 0 (min xpos width))
            :mousey (max 0 (min ypos height)))
      (after-paint layer))))

(defn set-layer! [mainobj window]
  (let [layer (SkiaLayerD3D12. nil nil nil (LayerD3D12.))]
    (.attach layer window)
    (.reconfigure (slayer layer))
    (.resize layer (.getWidth (.getContentRect window)) (.getHeight (.getContentRect window)))
    (set-layer mainobj layer)))

(defn accepter [window mainobj]
  (fn accept [listener e]
    (let [layer (get-layer mainobj)]
      (cond
       (instance? EventWindowScreenChange e)
       (do (.reconfigure (slayer layer))
           (set-scale mainobj (.getScale (.getScreen window)))
           (accept listener (EventWindowResize.
                             (.getWidth(.getWindowRect window)),
                             (.getHeight(.getWindowRect window)),
                             (.getWidth(.getContentRect window)),
                             (.getHeight(.getContentRect window)))))

       ;; (instance? EventWindowResize e)
       ;; _width = (int) (er.getContentWidth() / _scale);
       ;; _height = (int) (er.getContentHeight() / _scale);
       ;; _layer.resize(er.getContentWidth(), er.getContentHeight());
       ;; paint();
       ;; (paint layer width height scale xpos ypos)

       ;; (instance? EventMouseMove e)
       ;; _xpos = (int) (((EventMouseMove) e).getX() / _scale);
       ;; _ypos = (int) (((EventMouseMove) e).getY() / _scale);

       (instance? EventFrame e)
       (do (paint layer mainobj)
           (.requestFrame window))

       (instance? EventWindowCloseRequest e)
       (do (.close layer)
           (.close window)
           (App/terminate))))))

(defn main [hwnd]
  {:pre [(integer? hwnd)]}
  (App/init)
  (let [window (doto (App/makeWindow)
                 (.winSetParent hwnd))
        mainobj (Main. 0 0 1 720 405 nil)
        accept (accepter window mainobj)
        listener (reify Consumer (accept [this e] (accept this e)))]
    (.setEventListener window  listener)
    (set-layer! mainobj window)
    (let [scale (.getScale (.getScreen window))]
      (.setWindowSize window (int (* 1440 scale)) (int (* 810 scale)))
      (.setWindowPosition window (int (* 240 scale)) (int (* 135 scale)))
      (.setVisible window true)
      (accept listener EventWindowScreenChange/INSTANCE)
      (.requestFrame window)
      (App/start))))
