(ns picker.window
  (:require
   [io.github.humbleui.window :as hui.win]
   [io.github.humbleui.macro :as macro])
  (:import
   [io.github.humbleui.window Window]
   [io.github.humbleui.jwm App Event EventWindowCloseRequest EventWindowScreenChange EventWindowResize EventFrame LayerD3D12 ZOrder]
   [io.github.humbleui.skija BackendRenderTarget ColorSpace DirectContext FramebufferFormat PixelGeometry Surface SurfaceColorFormat SurfaceOrigin SurfaceProps]
   [java.util.function Consumer]))

(defmacro prln []
  `(println ~(:line (meta &form))))


;; source based on https://github.com/HumbleUI/HumbleUI
(defn make
  ":on-close         (fn [window])
   :on-screen-change (fn [window])
   :on-resize        (fn [window {:keys [window-width window-height content-width content-height]}])
   :on-paint         (fn [window canvas])
   :on-event         (fn [window event])"
  [{:keys [on-close on-screen-change on-resize on-paint on-event]}]
  (let [jwm-window (App/makeWindow)
        jwm-layer  (LayerD3D12.)
        _          (.attach jwm-layer jwm-window)
        *context   (volatile! (DirectContext/makeDirect3D
                               (.getAdapterPtr jwm-layer)
                               (.getDevicePtr jwm-layer)
                               (.getQueuePtr jwm-layer)))
        *target    (volatile! nil)
        *surface   (volatile! nil)
        *window    (volatile! nil)
        paint      (when on-paint
                     (fn []
                       (when-some [window @*window]
                         (vswap! *target  #(or % (BackendRenderTarget/makeDirect3D
                                                  (.getWidth jwm-layer)
                                                  (.getHeight jwm-layer)
                                                  ;; 0 8 0
                                                  (.nextDrawableTexturePtr jwm-layer)
                                                  (.getFormat jwm-layer)
                                                  (.getSampleCount jwm-layer)
                                                  (.getLevelCount jwm-layer))))
                         (vswap! *surface #(or % (Surface/makeFromBackendRenderTarget
                                                  @*context
                                                  @*target
                                                  SurfaceOrigin/BOTTOM_LEFT
                                                  SurfaceColorFormat/RGBA_8888
                                                  (ColorSpace/getSRGB)
                                                  (SurfaceProps. PixelGeometry/RGB_H))))
                         ;; (prln)
                         #_(on-paint window (.getCanvas @*surface))
                         #_(.flush @*surface)
                         (.submit @*context false)
                         (.swapBuffers jwm-layer)
                         #_(.close @*surface)
                         #_(.close @*target))))
        listener   (fn listener [e]
                     (when on-event
                       (on-event @*window e))
                     (cond
                       (instance? EventWindowCloseRequest e)
                       (do
                         (println "closing")
                         (when on-close (on-close @*window))
                         (when (some? @*context)
                           (vswap! *context #(do (macro/doto-some % .abandon .close)
                                                 nil)))
                         ;; (vswap! *surface #(do (macro/doto-some % .close) nil))
                         ;; (vswap! *target #(do (macro/doto-some % .close) nil))
                         (.close jwm-layer)
                         (.close jwm-window)
                         (vreset! *window nil))

                       #_(instance? EventWindowScreenChange e)
                       #_(do
                         (when on-screen-change (on-screen-change @*window))
                         (.reconfigure jwm-layer)
                         (let [outer  (.getWindowRect jwm-window)
                               inner  (.getContentRect jwm-window)
                               resize (EventWindowResize. (.getWidth outer)
                                                          (.getHeight outer)
                                                          (.getWidth inner)
                                                          (.getHeight inner))]
                           (listener resize)))

                       (instance? EventFrame e)
                       (do
                         (when paint (paint))
                         (hui.win/request-frame @*window))
                       (instance? EventWindowResize e)
                       (do
                         #_#_#_#_(prln)
                         (when on-resize
                           (on-resize
                            @*window
                            {:window-width   (.getWindowWidth e)
                             :window-height  (.getWindowHeight e)
                             :content-width  (.getContentWidth e)
                             :content-height (.getContentHeight e)}))
                         (.submit @*context true)
                         (.resize jwm-layer (.getContentWidth e) (.getContentHeight e))
                         ;; (vswap! *surface #(do (macro/doto-some % .close) nil))
                         ;; (vswap! *target #(do (macro/doto-some % .close) nil))
                         ;; (vswap! *context #(do (macro/doto-some % .abandon .close) nil))
                         (when paint (paint)))

                       (instance? EventFrame e)
                       (when paint (paint))))
        _          (.setEventListener jwm-window (reify Consumer (accept [_this e] (listener e))))
        window     (Window. jwm-window jwm-layer listener)]
    (vreset! *window window)
    (listener EventWindowScreenChange/INSTANCE)
    window))

(comment
  (App/init)
  (def window (App/makeWindow))
  (alter-var-root #'window (fn [window]
                             (hui.win/close window)
                             (make {})))

  (.setVisible (:jwm-window window) true)
  (.setVisible (:jwm-window window) false)

  ;; (future (App/start))

  ;;
  )
