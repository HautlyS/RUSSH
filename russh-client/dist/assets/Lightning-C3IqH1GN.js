import{d as b,z as R,o as A,a as _,w as T,B as C,e as E}from"./vue-vendor-DDhLe2KT.js";import{i as P}from"./index-Bx4hJxvb.js";const z=`
attribute vec2 aPosition;
void main() {
  gl_Position = vec4(aPosition, 0.0, 1.0);
}
`,I=`
precision mediump float;
uniform vec2 iResolution;
uniform float iTime;
uniform float uHue;
uniform float uXOffset;
uniform float uSpeed;
uniform float uIntensity;
uniform float uSize;

#define OCTAVE_COUNT 10

vec3 hsv2rgb(vec3 c) {
    vec3 rgb = clamp(abs(mod(c.x * 6.0 + vec3(0.0,4.0,2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
    return c.z * mix(vec3(1.0), rgb, c.y);
}

float hash11(float p) {
    p = fract(p * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}

float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

mat2 rotate2d(float theta) {
    float c = cos(theta);
    float s = sin(theta);
    return mat2(c, -s, s, c);
}

float noise(vec2 p) {
    vec2 ip = floor(p);
    vec2 fp = fract(p);
    float a = hash12(ip);
    float b = hash12(ip + vec2(1.0, 0.0));
    float c = hash12(ip + vec2(0.0, 1.0));
    float d = hash12(ip + vec2(1.0, 1.0));
    
    vec2 t = smoothstep(0.0, 1.0, fp);
    return mix(mix(a, b, t.x), mix(c, d, t.x), t.y);
}

float fbm(vec2 p) {
    float value = 0.0;
    float amplitude = 0.5;
    for (int i = 0; i < OCTAVE_COUNT; ++i) {
        value += amplitude * noise(p);
        p *= rotate2d(0.45);
        p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
    vec2 uv = fragCoord / iResolution.xy;
    uv = 2.0 * uv - 1.0;
    uv.x *= iResolution.x / iResolution.y;
    uv.x += uXOffset;
    
    uv += 2.0 * fbm(uv * uSize + 0.8 * iTime * uSpeed) - 1.0;
    
    float dist = abs(uv.x);
    vec3 baseColor = hsv2rgb(vec3(uHue / 360.0, 0.7, 0.8));
    vec3 col = baseColor * pow(mix(0.0, 0.07, hash11(iTime * uSpeed)) / dist, 1.0) * uIntensity;
    col = pow(col, vec3(1.0));
    fragColor = vec4(col, 1.0);
}

void main() {
    mainImage(gl_FragColor, gl_FragCoord.xy);
}
`,U=b({__name:"Lightning",props:{hue:{default:230},xOffset:{default:0},speed:{default:1},intensity:{default:1},size:{default:1}},setup(L){const r=L,f=R("canvasRef");let m=0,e=null,o=null,g=0,w=null;const x=(t,i)=>{if(!e)return null;const n=e.createShader(i);return n?(e.shaderSource(n,t),e.compileShader(n),e.getShaderParameter(n,e.COMPILE_STATUS)?n:(console.error("Shader compile error:",e.getShaderInfoLog(n)),e.deleteShader(n),null)):null},y=()=>{const t=f.value;if(!t)return;const i=()=>{const d=t.getBoundingClientRect(),h=window.devicePixelRatio||1;let a=d.width,s=d.height,c=t.parentElement;for(;c&&(!a||!s);){if(c.offsetWidth&&c.offsetHeight){a=c.offsetWidth,s=c.offsetHeight;break}c=c.parentElement}(!a||!s)&&(a=window.innerWidth,s=window.innerHeight),a=Math.max(a,300),s=Math.max(s,300),t.width=a*h,t.height=s*h,t.style.width="100%",t.style.height="100%",t.style.display="block",t.style.position="absolute",t.style.top="0",t.style.left="0"};if(i(),window.addEventListener("resize",i),w=()=>window.removeEventListener("resize",i),e=t.getContext("webgl"),!e){console.error("WebGL not supported");return}const n=x(z,e.VERTEX_SHADER),l=x(I,e.FRAGMENT_SHADER);if(!n||!l||(o=e.createProgram(),!o))return;if(e.attachShader(o,n),e.attachShader(o,l),e.linkProgram(o),!e.getProgramParameter(o,e.LINK_STATUS)){console.error("Program linking error:",e.getProgramInfoLog(o));return}e.useProgram(o);const p=new Float32Array([-1,-1,1,-1,-1,1,-1,1,1,-1,1,1]),v=e.createBuffer();e.bindBuffer(e.ARRAY_BUFFER,v),e.bufferData(e.ARRAY_BUFFER,p,e.STATIC_DRAW);const u=e.getAttribLocation(o,"aPosition");return e.enableVertexAttribArray(u),e.vertexAttribPointer(u,2,e.FLOAT,!1,0,0),g=performance.now(),S(),()=>{window.removeEventListener("resize",i)}},S=()=>{if(!e||!o||!f.value)return;const t=f.value,i=t.getBoundingClientRect();(t.width!==i.width||t.height!==i.height)&&(t.width=i.width,t.height=i.height,t.style.width=i.width+"px",t.style.height=i.height+"px"),e.viewport(0,0,t.width,t.height);const n=e.getUniformLocation(o,"iResolution"),l=e.getUniformLocation(o,"iTime"),p=e.getUniformLocation(o,"uHue"),v=e.getUniformLocation(o,"uXOffset"),u=e.getUniformLocation(o,"uSpeed"),d=e.getUniformLocation(o,"uIntensity"),h=e.getUniformLocation(o,"uSize");e.uniform2f(n,t.width,t.height);const a=performance.now();e.uniform1f(l,(a-g)/1e3),e.uniform1f(p,r.hue),e.uniform1f(v,r.xOffset),e.uniform1f(u,r.speed),e.uniform1f(d,r.intensity),e.uniform1f(h,r.size),e.drawArrays(e.TRIANGLES,0,6),m=requestAnimationFrame(S)};return A(()=>{y()}),_(()=>{m&&cancelAnimationFrame(m),w?.(),e&&o&&e.deleteProgram(o),e=null,o=null}),T(()=>[r.hue,r.xOffset,r.speed,r.intensity,r.size],()=>{}),(t,i)=>(E(),C("canvas",{ref_key:"canvasRef",ref:f,class:"w-full h-full block mix-blend-screen relative"},null,512))}}),B=P(U,[["__scopeId","data-v-043dde91"]]);export{B as L};
