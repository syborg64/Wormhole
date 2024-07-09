# Specifications Non-Techniques

***Obligatoire :***

## Évaluer et Intégrer les nouvelles technologies
Nous utiliserons une stack technique **récente**, avec une communauté active et **axée performance / sécurité**.
> [!TIP] Rust nous parrait le choix de langage le plus approprié.
> Nous resterons à l'écoute des évolutions de ce langage comme des autres pour ajuster nos choix.

**Nous suivrons l'apparition de nouvelles plateformes** et de leur pertinence pour une potentielle intégration native.
> [!NOTE] Les plateformes prioritaires sont actuellement :
> - Linux
> - Windows
> - Mac


**Nous explorerons les différents protocoles réseau qui pourraient nous servir au mieux**, tant pour leur vitesse que pour leur fiabilité.   
Cela va de soit aussi pour les protocoles d'accès.
> [!IMPORTANT] Dans le cadre de l'EIP, nous utiliserons les protocoles natifs pour les systèmes précédement cités.
> Mais nous sommes ouverts à l'intégration future de protocoles ouverts pour étendre nos compatibilités.


Nous tiendrons un environnement de développement à jour pour accélérer les temps d’itérations.

<br>

___

***Optionnels :***
## Protéger et améliorer notre technologie
**Nous sélectionnerons et appliquerons une licence de développement open source qui servira au mieux notre projet.**   
> [!NOTE] Nous réfléchissons à des licences ouvertes pour les particuliers et payantes pour l'usage commercial / d'entreprises.   
> Ces licences ont le bénéfice de ne pas entraver la démocratisation du projet tout en ouvrant la possibilité de le rentabiliser.


Nous validerons soigneusement nos dépendances afin de :
- Respecter les licences
- Respecter nos objectifs de légereté, performance et multiplateforme
- Limiter notre surface d'attaque

## Entretenir les contributions par la communauté
**Nous voulons obtenir rapidement le soutien de la communauté.**  
Cela passe par plusieurs mesures :

### - Utilisation plaisante et accessible
Bien que notre outil reste technique et qu'il ne disposera pas de notion "UI/UX" à proprement parler, nous ferons de notre mieux pour le rendre intuitif dès la première utilisation, et surtout pour les besoins courrants et simples. Tout cela afin de ne pas décourager les personnes pouvant s'intérésser au projet.
> [!TIP] L'environnement de développement Rust est une bonne inspiration.
> Rust est intrasequement une notion technique, mais la "developer experience" est une préoccupation.
> - La documentation est claire.
> - Le compilateur détaille et explique les erreurs ou warnings de manière claire.
> - Vient avec une suite d'outils (formatting, cross-compilation, intégration Visual Studio Code...)
>
>
> Tout ceci joue probablement une grande part dans la popularité de Rust, et est inspirant pour un projet comme le notre.

> [!CAUTION] Notre EIP reste un EIP technique.
> L'expérience utilisateur fera partie de nos préoccupations car elle constitue une bonne stratégie, mais elle reste au second plan face aux objectifs techniques.   
> Les objectifs "qualité" (documentation claire, CLI bien pensée) seront bien sur intégrés, mais les objectifs additionnels (ex. suite d'outils) ne feront pas partie des sprints ou objectifs de l'EIP.

### - Clareté technique
Nous ciblerons un public qui souhaite des améliorations au projet et possède la volonté de les faire.   
Cela implique :
- Maintenir une documentation technique publique et claire, pour faciliter le développement par des tiers.
- Notre projet sera publique sur GitHub et incitera à la contribution.
- La RoadMap sera publiée pour donner à chaque contributeur potentiel une idée de l'avancement et de l'activité du projet.
- Dans la mesure du possible, nous parlerons de notre projet sur des groupes internet centrés autour du sujet (Reddit, Discord...)
