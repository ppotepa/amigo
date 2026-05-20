import { type BodyPart } from "./types";

export const anatomyRig: BodyPart = {
  id: "body",
  type: "group",
  anchor: { x: 0, y: 0, z: 0 },
  children: [
    {
      id: "head",
      type: "group",
      anchor: { x: 0, y: 0, z: 0 },
      children: [
        {
          id: "face",
          type: "mass",
          anchor: { x: 0, y: -16, z: 0 },
          geometry: { strategy: "head.faceMorph" },
          visibilityMode: "always",
          fusionGroup: "skin",
          renderPass: "midMass",
          render: {
            material: "skin",
            zIndex: 20,
            primaryLighting: true,
            debugSamplePoints: true,
            outline: {
              contourClassName: "partOutline",
              contourZIndex: 100,
              silhouetteZIndex: 90,
            },
            outputs: [
              { kind: "path", role: "fill", className: "surfaceBase" },
              { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
              { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
              { kind: "path", role: "wire", className: "wirePath" },
            ],
          },
        },
        {
          id: "nose",
          type: "mass",
          anchor: { x: 0, y: 8, z: 42 },
          geometry: { strategy: "head.noseMorph" },
          visibilityMode: "not-back",
          fusionGroup: "skin",
          renderPass: "nearMass",
          render: {
            material: "skin",
            zIndex: 30,
            outline: {
              contourClassName: "partOutline strong",
              contourZIndex: 101,
              silhouetteZIndex: 91,
            },
            outputs: [
              { kind: "path", role: "fill", className: "svgNose" },
              { kind: "path", role: "wire", className: "wirePath" },
            ],
          },
          children: [
            {
              id: "noseHighlight",
              type: "detail",
              anchor: { x: 0, y: 0, z: 50 },
              geometry: {
                strategy: "head.noseHighlight",
                input: { parent: true },
              },
              visibilityMode: "front-side",
              renderPass: "detail",
              render: {
                zIndex: 60,
                outputs: [{ kind: "path", role: "detail", layer: "rig", className: "noseHighlight" }],
              },
            },
            {
              id: "nostril",
              type: "detail",
              anchor: { x: 0, y: 18, z: 48 },
              geometry: {
                strategy: "head.nostril",
                input: { parent: true },
              },
              visibilityMode: "front-side",
              renderPass: "detail",
              render: {
                zIndex: 61,
                outputs: [
                  {
                    kind: "ellipse",
                    role: "detail",
                    layer: "rig",
                    className: "nostril",
                    metaAttrs: ["cx", "cy", "rx", "ry"],
                  },
                ],
              },
            },
          ],
        },
        {
          id: "earRight",
          type: "mass",
          anchor: { x: 76, y: 0, z: -8 },
          geometry: { strategy: "head.earMorph", side: "right" },
          visibilityMode: "always",
          fusionGroup: "skin",
          renderPass: "midMass",
          render: {
            material: "skin",
            zIndex: 25,
            outline: {
              contourClassName: "partOutline",
              contourZIndex: 102,
              silhouetteZIndex: 92,
            },
            outputs: [
              { kind: "path", role: "fill", className: "surfaceBase" },
              { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
              { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
              { kind: "path", role: "wire", className: "wirePath" },
            ],
          },
          children: [
            {
              id: "earRightInner",
              type: "detail",
              anchor: { x: 78, y: 0, z: -4 },
              geometry: {
                strategy: "head.earInnerCurve",
                side: "right",
                input: { parent: true },
              },
              visibilityMode: "front-side",
              renderPass: "detail",
              render: {
                zIndex: 62,
                outputs: [{ kind: "path", role: "detail", layer: "rig", className: "detailContour" }],
              },
            },
          ],
        },
        {
          id: "earLeft",
          type: "mass",
          anchor: { x: -76, y: 0, z: -8 },
          geometry: { strategy: "head.earMorph", side: "left" },
          visibilityMode: "always",
          fusionGroup: "skin",
          renderPass: "midMass",
          render: {
            material: "skin",
            zIndex: 25,
            outline: {
              contourClassName: "partOutline",
              contourZIndex: 103,
              silhouetteZIndex: 93,
            },
            outputs: [
              { kind: "path", role: "fill", className: "surfaceBase" },
              { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
              { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
              { kind: "path", role: "wire", className: "wirePath" },
            ],
          },
          children: [
            {
              id: "earLeftInner",
              type: "detail",
              anchor: { x: -78, y: 0, z: -4 },
              geometry: {
                strategy: "head.earInnerCurve",
                side: "left",
                input: { parent: true },
              },
              visibilityMode: "front-side",
              renderPass: "detail",
              render: {
                zIndex: 62,
                outputs: [{ kind: "path", role: "detail", layer: "rig", className: "detailContour" }],
              },
            },
          ],
        },
        {
          id: "mouth",
          type: "detail",
          anchor: { x: 0, y: 52, z: 36 },
          geometry: {
            strategy: "head.mouthLine",
            input: { sourcePartId: "face" },
          },
          visibilityMode: "not-back",
          renderPass: "detail",
          render: {
            zIndex: 55,
            outputs: [{ kind: "path", role: "detail", layer: "rig", className: "mouthLine" }],
          },
        },
      ],
    },
  ],
};
