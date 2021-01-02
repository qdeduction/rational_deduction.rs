// file: src/lib.rs
// authors: Brandon H. Gomes

//! Rational Deduction Algorithms

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use {
    core::{convert::TryFrom, iter::FromIterator},
    exprz::{Expr, Expression},
};

/// Package Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Ratio Trait
pub trait Ratio<V>
where
    Self: Into<RatioPair<V>>,
{
    /// Create a new ratio from two base type elements.
    fn new(top: V, bot: V) -> Self;

    /// Get reference to top and bottom of ratio.
    fn cases(&self) -> RatioPairRef<'_, V>;

    /// Get the top element.
    #[inline]
    fn top(self) -> V {
        self.into().top
    }

    /// Get reference to the top element.
    #[inline]
    fn top_ref(&self) -> &V {
        self.cases().top
    }

    /// Get the bottom element.
    #[inline]
    fn bot(self) -> V {
        self.into().bot
    }

    /// Get reference to the bottom element.
    #[inline]
    fn bot_ref(&self) -> &V {
        self.cases().bot
    }

    /// Convert from a `RatioPair`.
    #[inline]
    fn from_pair(pair: RatioPair<V>) -> Self {
        Self::new(pair.top, pair.bot)
    }

    /// Reverse a `Ratio`.
    #[inline]
    fn reverse(self) -> Self {
        let ratio = self.into();
        Self::new(ratio.bot, ratio.top)
    }

    /// Get the default ratio.
    #[inline]
    fn default() -> Self
    where
        V: Default,
    {
        Self::from_pair(Default::default())
    }

    /// Clone a `Ratio`.
    #[inline]
    fn clone(&self) -> Self
    where
        V: Clone,
    {
        Self::new(self.top_ref().clone(), self.bot_ref().clone())
    }

    /// Check if two `Ratio`s are equal.
    #[inline]
    fn eq<RV, R>(&self, other: &R) -> bool
    where
        R: Ratio<RV>,
        V: PartialEq<RV>,
    {
        self.eq_by(other, PartialEq::eq)
    }

    /// Check if two `Ratio`s are equal given the comparison function.
    fn eq_by<RV, R, F>(&self, other: &R, mut eq: F) -> bool
    where
        R: Ratio<RV>,
        F: FnMut(&V, &RV) -> bool,
    {
        eq(self.top_ref(), other.top_ref()) && eq(self.bot_ref(), other.bot_ref())
    }

    /// Compose two ratios using the ratio monoid multiplication algorithm.
    #[inline]
    fn pair_compose<T>(top: Self, bot: Self) -> Self
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        V::Item: PartialEq,
    {
        Self::pair_compose_by(top, bot, PartialEq::eq)
    }

    /// Compose two ratios using the ratio monoid multiplication algorithm.
    fn pair_compose_by<F>(top: Self, bot: Self, eq: F) -> Self
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        F: FnMut(&V::Item, &V::Item) -> bool,
    {
        let top = top.into();
        let bot = bot.into();
        let (lower, upper) = util::multiset_symmetric_difference_by::<_, V, _>(
            top.bot,
            bot.top.into_iter().collect(),
            eq,
        );
        Self::new(
            upper.chain(top.top).collect(),
            lower.into_iter().chain(bot.bot).collect(),
        )
    }

    /// Fold a collection of ratios using [`pair_compose`].
    ///
    /// [`pair_compose`]: trait.Ratio.html#method.pair_compose
    #[inline]
    fn compose<I>(ratios: I) -> Self
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        V::Item: PartialEq,
        I: IntoIterator<Item = Self>,
    {
        Self::compose_by(ratios, PartialEq::eq)
    }

    /// Fold a collection of ratios using [`pair_compose_by`].
    ///
    /// [`pair_compose_by`]: trait.Ratio.html#method.pair_compose_by
    fn compose_by<I, F>(ratios: I, mut eq: F) -> Self
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        I: IntoIterator<Item = Self>,
        F: FnMut(&V::Item, &V::Item) -> bool,
    {
        let mut iter = ratios.into_iter();
        iter.next()
            .map(move |r| iter.fold(r, move |t, b| Self::pair_compose_by(t, b, &mut eq)))
            .unwrap_or_else(|| Self::new(V::from_iter(None), V::from_iter(None)))
    }

    /// Check if there would be any cancellation if you composed the two elements.
    #[inline]
    fn has_cancellation(top: &Self, bot: &Self) -> bool
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        V::Item: PartialEq,
    {
        Self::has_cancellation_by(top, bot, PartialEq::eq)
    }

    /// Check if there would be any cancellation if you composed the two elements.
    fn has_cancellation_by<F>(top: &Self, bot: &Self, eq: F) -> bool
    where
        V: IntoIterator + FromIterator<<V as IntoIterator>::Item>,
        F: FnMut(&V::Item, &V::Item) -> bool,
    {
        let _ = (top, bot, eq);
        /*
        let top = top.cases();
        let bot = bot.cases();
        util::has_intersection_by(top.bot, bot.top.into_iter().collect(), &mut eq)
            || util::has_intersection_by(top.top, bot.bot.into_iter().collect(), &mut eq)
        */
        todo!()
    }
}

/// Ratio Reference Type
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RatioPairRef<'v, V> {
    /// Top of the ratio
    pub top: &'v V,

    /// Bottom of the ratio
    pub bot: &'v V,
}

/// Canonical Ratio Type
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RatioPair<V> {
    /// Top of the ratio
    pub top: V,

    /// Bottom of the ratio
    pub bot: V,
}

impl<V> Ratio<V> for RatioPair<V> {
    #[inline]
    fn new(top: V, bot: V) -> Self {
        Self { top, bot }
    }

    #[inline]
    fn cases(&self) -> RatioPairRef<'_, V> {
        RatioPairRef {
            top: &self.top,
            bot: &self.bot,
        }
    }
}

impl<V> Into<RatioPair<V>> for (V, V) {
    #[inline]
    fn into(self) -> RatioPair<V> {
        RatioPair::new(self.0, self.1)
    }
}

impl<V> Ratio<V> for (V, V) {
    #[inline]
    fn new(top: V, bot: V) -> Self {
        (top, bot)
    }

    #[inline]
    fn cases(&self) -> RatioPairRef<'_, V> {
        RatioPairRef {
            top: &self.0,
            bot: &self.1,
        }
    }
}

impl<E> From<RatioPair<E::Group>> for Expr<E>
where
    E: Expression,
    E::Group: IntoIterator<Item = E> + FromIterator<E>,
{
    /// Convert a `RatioPairExpr<E>` into an `Expr<E>` by forgetting the shape of the underlying
    /// expression.
    #[inline]
    fn from(ratio: RatioPair<E::Group>) -> Self {
        Self::Group(
            Some(E::from_group(ratio.top))
                .into_iter()
                .chain(Some(E::from_group(ratio.bot)))
                .collect(),
        )
    }
}

impl<E> TryFrom<Expr<E>> for RatioPair<E::Group>
where
    E: Expression,
    E::Group: IntoIterator<Item = E>,
{
    type Error = expr::RatioPairFromExprError;

    /// Parse an `Expr<E>` into a `RatioPairExpr<E>` if it has the correct shape.
    fn try_from(expr: Expr<E>) -> Result<Self, Self::Error> {
        match expr {
            Expr::Atom(_) => Err(expr::RatioPairFromExprError::NotGroup),
            Expr::Group(group) => {
                let mut iter = group.into_iter();
                if let (Some(top), Some(bot), None) = (iter.next(), iter.next(), iter.next()) {
                    match (top.into(), bot.into()) {
                        (Expr::Group(top), Expr::Group(bot)) => Ok(Self { top, bot }),
                        (_, Expr::Group(_)) => Err(expr::RatioPairFromExprError::MissingTopGroup),
                        (Expr::Group(_), _) => Err(expr::RatioPairFromExprError::MissingBotGroup),
                        _ => Err(expr::RatioPairFromExprError::MissingTopBotGroup),
                    }
                } else {
                    Err(expr::RatioPairFromExprError::BadGroupShape)
                }
            }
        }
    }
}

/// Expression Ratio Module
pub mod expr {
    use {
        super::Ratio,
        core::{borrow::Borrow, iter::FromIterator},
        exprz::{iter::IteratorGen, ExprRef, Expression},
    };

    /// Conversion from `Expr` to `RatioPair` Error Type
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum RatioPairFromExprError {
        /// The expression is not a group.
        NotGroup,

        /// The expression has the wrong group shape.
        BadGroupShape,

        /// The top element of the group is not a group.
        MissingTopGroup,

        /// The bot element of the group is not a group.
        MissingBotGroup,

        /// The top and bot element of the group are not groups.
        MissingTopBotGroup,
    }

    /// Check if an `Expression` has the right shape to be a ratio.
    ///
    /// Use [`try_from`] to convert an `Expr<E>` to a `RatioPairExpr<E>`.
    ///
    /// [`try_from`]: ../struct.RatioPair.html#impl-TryFrom<Expr<E>>
    #[must_use]
    pub fn has_ratio_shape<E>(expr: &E) -> bool
    where
        E: Expression,
    {
        match expr.cases() {
            ExprRef::Group(group) => {
                let mut iter = group.iter();
                if let (Some(top), Some(bot), None) = (iter.next(), iter.next(), iter.next()) {
                    top.borrow().is_group() && bot.borrow().is_group()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Substitute an `Expression` into each `Atom` of `self`.
    #[inline]
    pub fn substitute<E, R, F>(ratio: R, mut f: F) -> R
    where
        E: Expression,
        E::Group: IntoIterator<Item = E> + FromIterator<E>,
        R: Ratio<E::Group>,
        F: FnMut(E::Atom) -> E,
    {
        let ratio = ratio.into();
        Ratio::new(
            ratio
                .top
                .into_iter()
                .map(|e| e.substitute(&mut f))
                .collect(),
            ratio
                .bot
                .into_iter()
                .map(|e| e.substitute(&mut f))
                .collect(),
        )
    }

    /// Evaluate a composition by performing each substitution and then composing ratios.
    #[inline]
    pub fn eval_composition<E, R, F, S, I>(terms: I) -> R
    where
        E: Expression + PartialEq,
        E::Group: IntoIterator<Item = E> + FromIterator<E>,
        R: Ratio<E::Group>,
        F: FnMut(E::Atom) -> E,
        S: AsMut<F>,
        I: IntoIterator<Item = (R, S)>,
    {
        Ratio::compose(
            terms
                .into_iter()
                .map(move |(r, mut s)| substitute(r, s.as_mut())),
        )
    }
}

/// Utilities
pub mod util {
    use {alloc::vec::Vec, core::iter::FromIterator, exprz::Expression};

    /// Compute the symmetric difference of two multisets.
    #[inline]
    pub fn multiset_symmetric_difference<L, OL>(
        left: L,
        right: Vec<L::Item>,
    ) -> (OL, impl Iterator<Item = L::Item>)
    where
        L: IntoIterator,
        L::Item: PartialEq,
        OL: FromIterator<L::Item>,
    {
        multiset_symmetric_difference_by(left, right, PartialEq::eq)
    }

    /// Compute the symmetric difference of two multisets.
    pub fn multiset_symmetric_difference_by<L, OL, F>(
        left: L,
        right: Vec<L::Item>,
        mut eq: F,
    ) -> (OL, impl Iterator<Item = L::Item>)
    where
        L: IntoIterator,
        OL: FromIterator<L::Item>,
        F: FnMut(&L::Item, &L::Item) -> bool,
    {
        // TODO: use bit-vector
        let right_len = right.len();
        let mut matched_indices = Vec::<bool>::with_capacity(right_len);
        matched_indices.resize(right_len, false);
        (
            left.into_iter()
                .filter(|l| {
                    (&right).iter().enumerate().all(|(i, r)| {
                        if eq(l, r) && !matched_indices[i] {
                            matched_indices[i] = true;
                            return false;
                        }
                        true
                    })
                })
                .collect(),
            right
                .into_iter()
                .zip(matched_indices)
                .filter_map(move |(r, m)| Some(r).filter(|_| !m)),
        )
    }

    /// See if the two multisets share any elements.
    #[inline]
    pub fn has_intersection<I>(left: I, right: Vec<&I::Item>) -> bool
    where
        I: IntoIterator,
        I::Item: PartialEq,
    {
        has_intersection_by(left, right, PartialEq::eq)
    }

    /// See if the two multisets share any elements.
    pub fn has_intersection_by<I, F>(left: I, right: Vec<&I::Item>, mut eq: F) -> bool
    where
        I: IntoIterator,
        F: FnMut(&I::Item, &I::Item) -> bool,
    {
        left.into_iter()
            .any(move |l| right.iter().all(|r| eq(&l, r)))
    }

    /// Generator for substitution using an iterator.
    #[inline]
    pub fn substitute_iter_on_atoms<'s, E, I>(iter: I, atom: E::Atom) -> E
    where
        E: 's + Expression,
        E::Atom: PartialEq,
        I: IntoIterator<Item = (&'s E::Atom, E)>,
    {
        iter.into_iter()
            .find(|(a, _)| **a == atom)
            .map(move |(_, t)| t)
            .unwrap_or_else(move || E::from_atom(atom))
    }
}
