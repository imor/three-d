in vec3 position;
in vec3 prev;
uniform mat4 model;
uniform mat4 viewProjection;
uniform vec2 resolution;
uniform float thickness;
out vec4 col;

void main() {
    mat4 modelViewProjection = viewProjection * model;
    vec4 clipPosition = modelViewProjection * vec4(position, 1.0);
    vec4 clipPrev = modelViewProjection * vec4(prev, 1.0);
    //perspective division
    vec2 ndcPosition = clipPosition.xy / clipPosition.w;
    vec2 ndcPrev = clipPrev.xy / clipPrev.w;
    //into screen space
    vec2 screenPosition = ndcPosition * resolution;
    vec2 screenPrev = ndcPrev * resolution;

    //vector along line segment
    vec2 lineSegVec = normalize(screenPosition - screenPrev);
    //vector normal to line segment vector
    vec2 normal = vec2(-lineSegVec.y, lineSegVec.x);

    //Shift vertex by half thickness
    float halfThickness = thickness / 2.0;
    screenPosition += normal * halfThickness;

    //back into ndcPosition
    ndcPosition = screenPosition / resolution;
    ndcPrev = screenPrev / resolution;
    //back into clip space
    clipPosition.xy = ndcPosition * clipPosition.w;
    clipPrev.xy = ndcPrev * clipPrev.w;

    gl_Position = clipPosition;

    col = vec4(1.0);
}
