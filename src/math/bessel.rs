//!
//! # Bessel functions
//! 
//! The Bessel functions are the solution to the [Bessel differential equation](https://en.wikipedia.org/wiki/Bessel_function#Spherical_Bessel_functions).
//! There are multiple variants of these solutions, and this sub-module provides functions for all of them.
//! 
//! ## First kind: J
//! 
//! The Jn(x) function is the simplest solution to Bessel's equation. The most generalized case is for
//! a real index `n` and a complex number `x`, which is possible with this crate. Additionally, two methods are
//! provided for this equation, one for a integer order (named `j`) and one for a real order (named `jf`). The integer
//! order one is limited but is faster than its counterpart. The real order function is slower, but can compute any
//! order. Note that `jf` will attempt to fall back on `j` when it finds an integer order.
//! 
//! ```rust
//! # use scilib::math::complex::Complex;
//! # use scilib::math::bessel::{ j, jf };
//! let c = Complex::from(-0.75, 3);
//! let res_i = j(c, 1);                // Faster for integer order
//! let res_f = jf(c, 1.5);             // Would also work with 1.0
//! ```
//! 
//! ## Second kind: Y
//! 
//! Similar to the first kind, the Y equation are solution of Bessel's equation with a singularity at the origin.
//! The Y function is itself based on the J function. The function is undefined for any integer order, in which
//! case the limit has to be taken.
//! 
//! ```rust
//! # use scilib::math::complex::Complex;
//! # use scilib::math::bessel::y;
//! let c = Complex::from(2, -1.2);
//! let res_f = y(c, 1.5);              // Not a problem
//! let res_i = y(c, 1);                // The function takes the limit in this case
//! ```
//! 
//! ## Modified first kind: I
//! 
//! Also known as the hyperbolic Bessel function of the first kind. Its definition is similar to J, but lacks the
//! alternating `(-1)^k` term in the sum.
//! 
//! ```rust
//! # use scilib::math::complex::Complex;
//! # use scilib::math::bessel::i;
//! let c = Complex::from(0.2, 1);
//! let res = i(c, -1.2);
//! ```
//! 
//! ## Modified second kind: K
//! 
//! Also known as the hyperbolic Bessel function of the second kind. Its definition is similar to Y, but lacks the
//! `cos(n*pi)`, and is normalized by `pi/2`.
//! 
//! ```rust
//! # use scilib::math::complex::Complex;
//! # use scilib::math::bessel::k;
//! let c = Complex::from(0, 7);
//! let res = k(c, 0);
//! ```
//! 
//! ## Hankel functions: H1 and H2
//! 
//! Hankel functions are two linearly independent solutions to Bessel's equation.
//! 
//! ```rust
//! # use scilib::math::complex::Complex;
//! # use scilib::math::bessel::{ hankel_first, hankel_second };
//! let c = Complex::from(-0.3, 1.52);
//! let res_1 = hankel_first(c, -2.3);
//! let res_2 = hankel_second(c, -2.3);
//! ```
//! 

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use std::f64::consts::{     // Using std lib constants
    PI,                     // Pi
    FRAC_PI_2               // Pi / 2
};

use super::{                // Using parts from the crate
    basic,                  // Basic functions
    complex::Complex        // Using Complex numbers
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// # Precision limit for Bessel computation
const PRECISION_CONVERGENCE: f64 = 1.0e-8;

/// # Limit when computing Bessel Y
const DISTANCE_Y_LIM: f64 = 0.001;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// # J Bessel function, integer index
///
/// `x` is the value to evaluate, and `n` the order of the function.
/// 
/// The J Bessel represent the first kind of Bessel function, and are solution to Bessel differential equation.
/// 
/// The bessel function depend on an infinite sum of terms; which we can't have.
/// The criterion chosen here is check each new term impacts the results significantly enough.
/// The default value selected in the program is defined by `const PRECISION_CONVERGENCE: f64 = 1.0e-8;`.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::j;
/// // Computing some example values
/// let res_00: Complex = j(0.0, 0);
/// let res_01: Complex = j(1.0, 0);
/// let res_10: Complex = j(0.0, 1);
/// let res_11: Complex = j(1.0, 1);
/// let res_20: Complex = j(0.0, 2);
/// let res_21: Complex = j(1.0, 2);
/// let res: Complex = j(5.2, 7);
/// 
/// // Comparing to tabulated data
/// assert_eq!(res_00, 1.0.into());
/// assert!((res_01.re - 0.7651976865).abs() < 1.0e-8);
/// assert_eq!(res_10, 0.0.into());
/// assert!((res_11.re - 0.44005058).abs() < 1.0e-8);
/// assert_eq!(res_20, 0.0.into());
/// assert!((res_21.re - 0.11490348).abs() < 1.0e-8);
/// assert!((res.re - 0.06544728).abs() < 1.0e-8);
/// 
/// // The function also handles negative orders
/// let pos1: Complex = j(3.2, 3);
/// let neg1: Complex = j(3.2, -3);
/// let pos2: Complex = j(2.45, 6);
/// let neg2: Complex = j(2.45, -6);
/// 
/// assert!(pos1 == -neg1);
/// assert!(pos2 == neg2);
/// 
/// // The input is treated as complex
/// let c: Complex = Complex::from(1, 2.5);
/// let res: Complex = j(c, 2);
/// ```
pub fn j<T: Into<Complex>>(x: T, n: i32) -> Complex {

    let np: i32 = n.abs();                                      // Getting the positive value of n
    let x2: Complex = x.into() / 2.0;                           // Halving x
    let mut k: i32 = 0;                                         // Order counter
    let mut d1: f64 = 1.0;                                      // First div
    let mut d2: f64 = basic::factorial(np as usize) as f64;     // Second div
    let mut sg: f64 = 1.0;                                      // Sign of the term

    let mut term: Complex = x2.powi(np) / d2;                   // The term at each step
    let mut res: Complex = Complex::default();                  // The result of the operation

    // If the first term is already too small we exit directly
    if term.modulus() < PRECISION_CONVERGENCE {
        return res;
    }

    // Computing the terms of the infinite series
    'convergence: loop {
        res += term;

        // If the changed compared to the final value is small we break
        if (term / res).modulus().abs() < PRECISION_CONVERGENCE {
            break 'convergence;
        }

        k += 1;                         // Incrementing value
        sg *= -1.0;                     // changing the sign of the term
        d1 *= k as f64;                 // Next value in the n! term
        d2 *= (np + k) as f64;          // Next value in the (n+k)! term
        term = sg * x2.powi(np + 2 * k) / (d1 * d2);
    }

    if n.is_negative() {
        (-1.0_f64).powi(np) * res
    } else {
        res
    }
}

/// # J Bessel function, real index
///
/// `x` is the value to evaluate, and `n` the order of the function.
/// 
/// Similar to the other J Bessel method, but this one allows the use of a real (float) index, rather
/// than an integer. This method is more costly to use than the other, and thus isn't recommended for
/// integer orders. The function tries to prevent this by trying to fall back on the integer order version
/// when possible, but could fail in edge cases.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::{ j, jf };
/// // This method allows the computation of real index for J
/// let res_pos: Complex = jf(1.0, 2.5);
/// let res_neg: Complex = jf(2.4, -1.75);
/// assert!((res_pos.re - 0.04949681).abs() < 1.0e-4);
/// assert!((res_neg.re - 0.11990699).abs() < 1.0e-4);
/// 
/// // We can also check that the results are coherent with integers
/// let res_i: Complex = j(0.75, 2);
/// let res_r: Complex = jf(0.75, 2);
/// assert!((res_i.re - res_r.re).abs() < 1.0e-4);
/// 
/// // Because results are Complex, negative numbers are allowed
/// let neg: Complex = jf(-0.75, 2.3);
/// let expected: Complex = Complex::from(0.0219887007, 0.030264850);
/// 
/// assert!((neg.re - expected.re).abs() < 1.0e-5 && (neg.im - expected.im).abs() < 1.0e-5);
/// 
/// // As for j, we can also use Complex numbers
/// let c: Complex = Complex::from(1.2, 0.5);
/// let res: Complex = jf(c, 1.5);
/// 
/// assert!((res.re - 0.3124202913).abs() < 1.0e-5 && (res.im - 0.1578998151) < 1.0e-5);
/// ```
pub fn jf<T, U>(x: T, order: U) -> Complex
    where T: Into<Complex>, U: Into<f64> {

    let n: f64 = order.into();
    // If the number passed in whole, we fall back on the other method instead
    if n.fract() == 0.0 {
        return j(x, n as i32);
    }

    let x2: Complex = x.into() / 2.0;           // Halving x
    let mut k: f64 = 0.0;                       // Order counter
    let mut d1: f64 = 1.0;                      // First div
    let mut d2: f64 = basic::gamma(n + 1.0);    // Second div
    let mut sg: f64 = 1.0;                      // Sign of the term

    let mut term: Complex = x2.powf(n) / d2;    // The term at each step
    let mut res: Complex = Complex::default();  // The result of the operation
    
    // If the first term is already too small we exit directly
    if term.modulus().abs() < PRECISION_CONVERGENCE {
        return res;
    }

    // Computing the terms of the infinite series
    'convergence: loop {
        res += term;

        // If the changed compared to the final value is small we break
        if (term / res).modulus().abs() < PRECISION_CONVERGENCE {
            break 'convergence;
        }

        k += 1.0;                       // Incrementing value
        sg *= -1.0;                     // changing the sign of the term
        d1 *= k;                        // Next value in the n! term
        d2 *= n + k;                    // Next value in the gamma(n+k+1) term
        term = sg * x2.powf(n + 2.0 * k) / (d1 * d2);
    }

    res
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// # Y Bessel function, real index
/// 
/// The Y Bessel represent the second kind of Bessel function, and are solution to Bessel differential equation,
/// in the case of a singularity at the origin.
/// 
/// `x` is the value to evaluate, and `n` the order of the function.
/// 
/// Because the function is not continuous for integer values of `n`, we need to compute the limit around these points.
/// We set the limit distance with `DISTANCE_Y_LIM`, compute the limit above and below the desired point and take the average.
/// We achieve precision under `1.0e-5` for non-integer`n`, and integer `n` using this approach.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::y;
/// let res_pos = y(1.0, 1.5);
/// let res_neg = y(1.0, -1.5);
/// 
/// assert!((res_pos.re - -1.10249557).abs() < 1.0e-5);
/// assert!((res_neg.re - -0.24029783).abs() < 1.0e-5);
/// 
/// // Values for integer n also works
/// let res_int_p = y(0.5, 1);
/// let res_int_n = y(0.5, -1);
/// 
/// assert!((res_int_p.re - -1.47147239).abs() < 1.0e-5);
/// assert!((res_int_n.re - 1.47147239).abs() < 1.0e-5);
/// 
/// // We can compute negative value with Y, the result is complex
/// let res_neg = y(-1.2, 3.1);
/// assert!((res_neg.re - 3.90596471).abs() < 1.0e-5 && (res_neg.im - -1.32157214).abs() < 1.0e-5);
/// 
/// // And we can use Complex as input
/// let c: Complex = Complex::from(-1.0, -0.5);
/// let res_c = y(c, 2.0);
/// 
/// assert!((res_c.re - -0.79108492).abs() < 1.0e-5 && (res_c.im - 0.60211151).abs() < 1.0e-5);
/// ```
pub fn y<T, U>(x: T, order: U) -> Complex
where T: Into<Complex> + Copy, U: Into<f64> {

    let n: f64 = order.into();

    // If n is whole, we have to take the limit, otherwise it's direct
    if n.fract() == 0.0 {
        (y(x, n + DISTANCE_Y_LIM) + y(x, n - DISTANCE_Y_LIM)) / 2.0
    } else {
        ((n * PI).cos() * jf(x, n) - jf(x, -n)) / (n * PI).sin()
    }
}

/// # I modified Bessel function
/// 
/// The I modified First Bessel function represent another kind of solution to the Bessel differential equation.
/// 
/// `x` is the value to evaluate, and `n` the order of the function.
/// 
/// We use a definition of I based on an infinite series (similar to J). This way, we ensure good precision in
/// the computation.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::i;
/// let res = i(1.2, 0);
/// assert!((res.re - 1.39373).abs() < 1.0e-4 && res.im == 0.0);
/// 
/// let c = Complex::from(-1.2, 0.5);
/// let r2 = i(c, -1.6);
/// assert!((r2.re - 0.549831).abs() < 1.0e-5 && (r2.im - -0.123202).abs() < 1.0e-5);
/// ```
pub fn i<T, U>(x: T, order: U) -> Complex
where T: Into<Complex>, U: Into<f64> + Copy {
    
    let n: f64 = order.into();

    let x2: Complex = x.into() / 2.0;           // Halving x
    let mut k: f64 = 0.0;                       // Order counter
    let mut d1: f64 = 1.0;                      // First div
    let mut d2: f64 = basic::gamma(n + 1.0);    // Second div

    let mut term: Complex = x2.powf(n) / d2;    // The term at each step
    let mut res: Complex = Complex::default();  // The result of the operation
    
    // If the first term is already too small we exit directly
    if term.modulus().abs() < PRECISION_CONVERGENCE {
        return res;
    }

    // Computing the terms of the infinite series
    'convergence: loop {
        res += term;

        // If the changed compared to the final value is small we break
        if (term / res).modulus().abs() < PRECISION_CONVERGENCE {
            break 'convergence;
        }

        k += 1.0;                       // Incrementing value
        d1 *= k;                        // Next value in the n! term
        d2 *= n + k;                    // Next value in the gamma(n+k+1) term
        term = x2.powf(n + 2.0 * k) / (d1 * d2);
    }

    res
}

/// # K modified Bessel function
/// 
/// The K modified Second Bessel function represent another kind of solution to the Bessel differential equation.
/// 
/// `x` is the value to evaluate, and `n` the order of the function.
/// 
/// The definition of K is similar to Y, but is based on I and not J.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::k;
/// let c1 = Complex::from(2, -1);
/// let res = k(c1, -3.5);
/// assert!((res.re - -0.32113627).abs() < 1.0e-5 && (res.im - 0.76751785).abs() < 1.0e-5);
/// 
/// // Similar to Y, we take the limit for integer orders
/// let c2 = Complex::from(-1.1, 0.6);
/// let res_i = k(c2, 1);
/// assert!((res_i.re - -1.6153940).abs() < 1.0e-5 && (res_i.im - -2.1056846).abs() < 1.0e-5);
/// ```
pub fn k<T, U>(x: T, order: U) -> Complex
where T: Into<Complex> + Copy, U: Into<f64> {

    let n: f64 = order.into();

    // If n is whole, we have to take the limit, otherwise it's direct
    if n.fract() == 0.0 {
        (k(x, n + DISTANCE_Y_LIM) + k(x, n - DISTANCE_Y_LIM)) / 2.0
    } else {
        (FRAC_PI_2 / (n * PI).sin()) * (i(x, -n) - i(x, n))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// # First Hankel function: H1
/// 
/// Computes the first kind of Hankel function, accepts complex input.
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::hankel_first;
/// let c1 = Complex::from(-1.1, 2.3);
/// let r1 = hankel_first(c1, 1);
/// assert!((r1.re - -0.0112027).abs() < 1.0e-5 && (r1.im - 0.0551947).abs() < 1.0e-5);
/// 
/// let c2 = Complex::from(5.2, -3);
/// let r2 = hankel_first(c2, -2.35);
/// assert!((r2.re - -4.2809477).abs() < 1.0e-5 && (r2.im - 3.2123502).abs() < 1.0e-5);
/// ```
pub fn hankel_first<T, U>(x: T, order: U) -> Complex
where T: Into<Complex> + Copy, U: Into<f64> {

    let n: f64 = order.into();
    let res_j = jf(x, n);
    let res_y = Complex::i() * y(x, n);

    res_j + res_y
}

/// # Second Hankel function: H2
/// 
/// Computes the second kind of Hankel function, accepts complex input.
/// We simplify the computation by simply conjugating the first kind
/// 
/// ```
/// # use scilib::math::complex::Complex;
/// # use scilib::math::bessel::hankel_second;
/// let c1 = Complex::from(-1.1, 2.3);
/// let r1 = hankel_second(c1, 1);
/// assert!((r1.re - -3.54421).abs() < 1.0e-5 && (r1.im - 2.2983539).abs() < 1.0e-5);
/// 
/// let c2 = Complex::from(5.2, -3);
/// let r2 = hankel_second(c2, -2.35);
/// assert!((r2.re - -0.0068184520).abs() < 1.0e-5 && (r2.im - -0.0193698).abs() < 1.0e-5);
pub fn hankel_second<T, U>(x: T, order: U) -> Complex
where T: Into<Complex> + Copy, U: Into<f64> {
    
    let n: f64 = order.into();
    let res_j = jf(x, n);
    let res_y = Complex::i() * y(x, n);

    res_j - res_y
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
