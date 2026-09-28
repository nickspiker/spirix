// nsobject_overhead.m  -  what idiomatic Objective-C costs for numerics.
//
// Same division, twice: once through the C structs of circle_divide.m, once through an NSObject with ARC properties. Every property read becomes an objc_msgSend, every result an objc_alloc_init plus an autorelease.
//
// Build and run:
//
//   clang -O2 -fobjc-arc -framework Foundation nsobject_overhead.m circle_divide.m -o nsobject_overhead
//   ./nsobject_overhead
//
// Measured 7.27 vs 124.17 ns/divide on an M-series Mac: 17x. Not a knock on the language - circle_divide.m is Objective-C too, and lands within one instruction of the hand-written IR. It is a knock on putting objects on a numeric hot path.

#import <Foundation/Foundation.h>
#include <stdint.h>
#include <stdbool.h>
#include <mach/mach_time.h>
typedef struct { int32_t real; int32_t imaginary; int16_t exponent; } SpxCircleF5E4;
bool circle_divide_struct(SpxCircleF5E4*, const SpxCircleF5E4*, const SpxCircleF5E4*);

@interface SpxCircle : NSObject
@property (nonatomic, assign) int32_t real;
@property (nonatomic, assign) int32_t imaginary;
@property (nonatomic, assign) int16_t exponent;
+ (SpxCircle *)divide:(SpxCircle *)n by:(SpxCircle *)d;
@end
@implementation SpxCircle
+ (SpxCircle *)divide:(SpxCircle *)n by:(SpxCircle *)d {
    SpxCircleF5E4 a = { n.real, n.imaginary, n.exponent };
    SpxCircleF5E4 b = { d.real, d.imaginary, d.exponent };
    SpxCircleF5E4 o;
    if (!circle_divide_struct(&o, &a, &b)) return nil;
    SpxCircle *r = [[SpxCircle alloc] init];
    r.real = o.real; r.imaginary = o.imaginary; r.exponent = o.exponent;
    return r;
}
@end

#define N 1024
int main(void) {
    mach_timebase_info_data_t tb; mach_timebase_info(&tb);
    SpxCircleF5E4 sa[N], sb[N];
    NSMutableArray *oa = [NSMutableArray array], *ob = [NSMutableArray array];
    uint64_t s = 0xA1B2C3D4E5F60718ull;
    #define NX (s = s*6364136223846793005ull + 1442695040888963407ull, (uint32_t)(s>>33))
    for (int i = 0; i < N; i++) {
        int32_t ar=NX, ai=NX, br=NX, bi=NX; int16_t e1=NX|1, e2=NX|1;
        sa[i]=(SpxCircleF5E4){ar,ai,e1}; sb[i]=(SpxCircleF5E4){br,bi,e2};
        SpxCircle *x=[[SpxCircle alloc] init]; x.real=ar; x.imaginary=ai; x.exponent=e1; [oa addObject:x];
        SpxCircle *y=[[SpxCircle alloc] init]; y.real=br; y.imaginary=bi; y.exponent=e2; [ob addObject:y];
    }
    double bs = 1e18, bo = 1e18;
    for (int t = 0; t < 12; t++) {
        uint64_t t0 = mach_absolute_time(); volatile int64_t acc = 0;
        for (int r = 0; r < 300; r++) for (int i = 0; i < N; i++) { SpxCircleF5E4 o; circle_divide_struct(&o,&sa[i],&sb[i]); acc += o.real; }
        double ns = (mach_absolute_time()-t0)*(double)tb.numer/tb.denom/(300.0*N);
        if (ns < bs) bs = ns;

        t0 = mach_absolute_time(); acc = 0;
        @autoreleasepool {
          for (int r = 0; r < 300; r++) for (int i = 0; i < N; i++) { SpxCircle *o=[SpxCircle divide:oa[i] by:ob[i]]; acc += o.real; }
        }
        ns = (mach_absolute_time()-t0)*(double)tb.numer/tb.denom/(300.0*N);
        if (ns < bo) bo = ns;
    }
    printf("  objc structs (C)      %6.2f ns/divide\n", bs);
    printf("  objc NSObject + ARC   %6.2f ns/divide   (%.1fx slower)\n", bo, bo/bs);
    return 0;
}
