export interface ImagePreviewProps {
  src: string;
  alt: string;
  width: number;
  height: number;
  zoom?: "fit" | number;
}
/** Content is never recolored or transformed by the theme. */
export function ImagePreview({ src, alt, width, height, zoom = "fit" }: ImagePreviewProps) {
  const fit = zoom === "fit";
  return <div className="n-image-viewport" tabIndex={0} role="region" aria-label={alt}>
    <div className="n-image-canvas" data-fit={fit} style={fit ? undefined : { width: width * zoom / 100, height: height * zoom / 100 }}>
      <img src={src} alt={alt} width={width} height={height} />
    </div>
  </div>;
}
